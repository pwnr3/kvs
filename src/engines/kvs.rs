use crate::{ErrorKind, KvsEngine, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter, SeekFrom};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are stored in a `HashMap` in memory and not persisted to disk.
///
/// Example:
///
/// ```rust
/// # use kvs::{KvsEngine, KvStore};
/// # use tempfile::TempDir;
/// let temp_dir = TempDir::new().unwrap();
/// let mut store = KvStore::open(temp_dir.path()).unwrap();
/// store.set("key".to_owned(), "value".to_owned());
/// let val = store.get("key".to_owned()).unwrap();
/// assert_eq!(val, Some("value".to_owned()));
/// ```
pub struct KvStore {
    db: Arc<DashMap<String, CommandPointer>>, // use DashMap to replace RwLock<HashMap<K, V>>
    path: Arc<PathBuf>,
    fname: AtomicU64,
    trash: AtomicU64,
    writer: Arc<Mutex<BufWriterWithPos<File>>>,
    readers: Arc<Mutex<HashMap<u64, BufReaderWithPos<File>>>>,
}

impl KvsEngine for KvStore {
    fn set(&self, key: String, val: String) -> Result<()> {
        let command = Command::Set {
            key: key.clone(),
            val: val,
        };
        let mut writer = self.writer.lock().unwrap();
        let pos = writer.pos;
        //write!(writer, "{}", serde_json::to_string(&command)?)?;
        serde_json::to_writer(&mut *writer, &command)?;
        //writer.write(serde_json::to_vec(&command)?.as_slice())?;
        writer.flush()?;
        let new_pos = writer.pos;
        drop(writer);

        let mut trash = self.trash.load(Ordering::SeqCst);
        if let Some(old_cmd) = self.db.insert(
            key,
            (self.fname.load(Ordering::SeqCst), pos..new_pos).into(),
        ) {
            trash += old_cmd.len;
            self.trash.store(trash, Ordering::SeqCst);
        }
        //drop(db); // if not explictly drop here, `compaction` test will fail

        if trash >= COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(rec) = self.db.get(&key) {
            //let f = File::open(&self.path.join(format!("{}.log", rec.fname.to_string())))?;
            //let mut reader = BufReader::new(f);
            let mut readers = self.readers.lock().unwrap();
            let reader = readers
                .get_mut(&rec.fname)
                .expect("Could not open log reader"); // will make `&self -> &mut self`
            reader.seek(SeekFrom::Start(rec.pos))?;
            let cmd_reader = reader.take(rec.len);
            if let Command::Set { val, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(val))
            } else {
                Err(ErrorKind::ReadFail)
            }
        } else {
            Ok(None)
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        if let Some((_, cmd)) = self.db.remove(&key) {
            let command = Command::Remove { key: key };
            let mut writer = self.writer.lock().unwrap();
            write!(writer, "{}", serde_json::to_string(&command)?)?;
            writer.flush()?;

            let trash = self.trash.load(Ordering::SeqCst) + cmd.len;
            self.trash.store(trash, Ordering::SeqCst);
            if trash >= COMPACTION_THRESHOLD {
                self.compact()?;
            }
            Ok(())
        } else {
            Err(ErrorKind::KeyNotFound)
        }
    }
}

impl KvStore {
    /// Open the KvStore at a given path. Return the KvStore.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();

        let list = sorted_gen_list(&path)?;
        let db: DashMap<String, CommandPointer> = DashMap::new();
        let mut readers = HashMap::new();
        let mut trash = 0;

        for &fname in &list {
            let f = File::open(&path.join(format!("{}.log", fname)))?;
            let reader = BufReader::new(&f);
            let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
            let mut pos = 0;
            while let Some(Ok(cmd)) = stream.next() {
                let new_pos = stream.byte_offset() as u64;
                match cmd {
                    Command::Set { key, .. } => {
                        if let Some(old_cmd) = db.insert(key, (fname, pos..new_pos).into()) {
                            trash += old_cmd.len;
                        }
                    }
                    Command::Remove { key } => {
                        if let Some((_, old_cmd)) = db.remove(&key) {
                            trash += old_cmd.len;
                        };
                        trash += new_pos - pos;
                    }
                }
                pos = new_pos;
            }
            readers.insert(fname, BufReaderWithPos::new(f));
        }

        let fname = list.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path, fname, &mut readers)?;

        Ok(KvStore {
            db: Arc::new(db),
            path: Arc::new(path),
            fname: AtomicU64::new(fname),
            trash: AtomicU64::new(trash),
            writer: Arc::new(Mutex::new(writer)),
            readers: Arc::new(Mutex::new(readers)),
        })
    }

    /// Compact log file.
    pub fn compact(&self) -> Result<()> {
        println!("Compaction start");
        let old_fname = self.fname.load(Ordering::SeqCst);
        // compact file
        let compact_fname = old_fname + 1;
        let mut readers = HashMap::new();
        let mut writer = new_log_file(&self.path, compact_fname, &mut readers)?;
        let mut pos = 0;
        let mut old_readers = self.readers.lock().unwrap();
        for mut cmd_pointer in self.db.iter_mut() {
            let reader = old_readers
                .get_mut(&cmd_pointer.fname)
                .expect("Could not open log reader");
            if reader.pos != cmd_pointer.pos {
                reader.seek(SeekFrom::Start(cmd_pointer.pos))?;
            }
            let mut cmd_reader = reader.take(cmd_pointer.len);
            let len = io::copy(&mut cmd_reader, &mut writer)?;
            cmd_pointer.fname = compact_fname;
            cmd_pointer.pos = pos;
            pos += len as u64;
        }
        writer.flush()?;

        // new writer
        self.fname.store(old_fname + 2, Ordering::SeqCst);
        *self.writer.lock().unwrap() = new_log_file(&self.path, old_fname + 2, &mut readers)?;

        //drop(old_readers); // program will stuck without this line, stupid mistake
        //// remove stale log
        //for fname in self.readers.lock().unwrap().keys() {
        //    fs::remove_file(self.path.join(format!("{}.log", fname)))?;
        //}
        //*self.readers.lock().unwrap() = readers;
        // remove stale log
        for fname in old_readers.keys() {
            println!("removing {}", fname);
            fs::remove_file(self.path.join(format!("{}.log", fname)))?;
        }
        *old_readers = readers;
        self.trash.store(0, Ordering::SeqCst);
        Ok(())
    }
}

impl Clone for KvStore {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            path: Arc::clone(&self.path),
            fname: AtomicU64::new(self.fname.load(Ordering::SeqCst)),
            trash: AtomicU64::new(self.trash.load(Ordering::SeqCst)),
            writer: Arc::clone(&self.writer),
            readers: Arc::clone(&self.readers),
        }
    }
}

fn sorted_gen_list(path: &Path) -> Result<Vec<u64>> {
    let mut list: Vec<u64> = fs::read_dir(path)?
        .flat_map(|res| -> Result<_> { Ok(res?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    list.sort_unstable();
    Ok(list)
}

fn new_log_file(
    path: &Path,
    fname: u64,
    readers: &mut HashMap<u64, BufReaderWithPos<File>>,
) -> Result<BufWriterWithPos<File>> {
    let file_name = path.join(format!("{}.log", fname));
    let f = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&file_name)?;
    let writer = BufWriterWithPos::new(f);
    readers.insert(fname, BufReaderWithPos::new(File::open(file_name)?));
    Ok(writer)
}

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, val: String },
    Remove { key: String },
}

#[derive(Debug, Clone)]
struct CommandPointer {
    fname: u64,
    pos: u64,
    len: u64,
}

impl From<(u64, Range<u64>)> for CommandPointer {
    fn from((fname, range): (u64, Range<u64>)) -> CommandPointer {
        CommandPointer {
            fname: fname,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}

struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

//impl<R: Read + Seek> Clone for BufReaderWithPos<R> {
//    fn clone(&self) -> Self {
//        Self {
//            reader: BufReader::new(*self.reader.get_ref()),
//            pos: self.pos,
//        }
//    }
//}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(inner: R) -> BufReaderWithPos<R> {
        BufReaderWithPos {
            reader: BufReader::new(inner),
            pos: 0,
        }
    }
}

impl<R> Read for BufReaderWithPos<R>
where
    R: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

//impl<W: Write + Seek> Clone for BufWriterWithPos<W> {
//    fn clone(&self) -> Self {
//        Self {
//            writer: BufWriter::new(*self.writer.get_ref()),
//            pos: self.pos,
//        }
//    }
//}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(inner: W) -> BufWriterWithPos<W> {
        BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos: 0,
        }
    }
}

impl<W> Write for BufWriterWithPos<W>
where
    W: Write + Seek,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}
