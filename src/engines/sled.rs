use crate::{ErrorKind, KvsEngine, Result};
use sled;
use std::path::PathBuf;

/// Wrapper of `sled::Db`
pub struct SledKvsEngine(sled::Db);

impl SledKvsEngine {
    /// Initialize
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        Ok(Self(sled::open(path.into().join("sled-db"))?))
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, val: String) -> Result<()> {
        self.0.insert(key, val.into_bytes())?;
        self.0.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        Ok(self
            .0
            .get(key)?
            .map(|val| AsRef::<[u8]>::as_ref(&val).to_vec())
            .map(String::from_utf8) // Option<Result>
            .transpose()?) // -> Result<Option>
                           //if let Ok(Some(val)) = self.0.get(key) {
                           //    let val = String::from_utf8(val.to_vec())?;
                           //    Ok(Some(val))
                           //} else {
                           //    Ok(None)
                           //}
    }

    fn remove(&mut self, key: String) -> Result<()> {
        self.0.remove(key)?.ok_or(ErrorKind::KeyNotFound)?;
        self.0.flush()?;
        Ok(())
    }
}
