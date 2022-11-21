use crate::{thread_pool::ThreadPool, Result};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Share thread in a pool.
pub struct SharedQueueThreadPool {
    sender: Option<mpsc::Sender<ThreadPoolMessage>>,
    pool: Vec<Worker>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));

        let mut pool = Vec::with_capacity(threads as usize);
        for i in 0..threads {
            pool.push(Worker::new(i, Arc::clone(&rx)));
        }

        Ok(Self {
            sender: Some(tx),
            pool: pool,
        })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .as_ref()
            .unwrap()
            .send(ThreadPoolMessage::RunJob(Box::new(job)))
            .expect("Fail to spawn thread");
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        if thread::panicking() {
            println!("Thread panic and quit");
        }

        for _ in 0..self.pool.len() {
            self.sender
                .as_ref()
                .unwrap()
                .send(ThreadPoolMessage::Shutdown)
                .expect("Fail to send shutdown message");
        }
        // explicitly drop sender before waiting for the threads to finish
        drop(self.sender.take());

        for worker in &mut self.pool {
            //println!("Shutting down worker {}", worker.id);
            if let Some(handle) = worker.handle.take() {
                handle.join().expect("Worker {worker.id} join fail");
            }
        }
    }
}

enum ThreadPoolMessage {
    RunJob(Job),
    Shutdown,
}

struct Worker {
    id: u32,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: u32, receiver: Arc<Mutex<mpsc::Receiver<ThreadPoolMessage>>>) -> Self {
        let handle = thread::spawn(move || loop {
            match receiver.lock() {
                Ok(receiver) => match receiver.recv() {
                    Ok(msg) => match msg {
                        ThreadPoolMessage::RunJob(job) => {
                            drop(receiver); // explicitly drop receiver
                            if let Err(e) = catch_unwind(AssertUnwindSafe(job)) {
                                println!("Worker {} panic: {:?}", id, e);
                            };
                        }
                        ThreadPoolMessage::Shutdown => {
                            //println!("Worker {} receive SHUTDOWN, closing", id);
                            return;
                        }
                    },
                    Err(e) => println!("Worker {} cannot receive message: {:?}", id, e),
                },
                Err(e) => println!("Worker {} lock failed: {:?}", id, e),
            }
        });
        Self {
            id: id,
            handle: Some(handle),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if thread::panicking() {
            println!("Worker {} panic and close", self.id);
            // instead of `catch_unwind` to reuse that thread,
            // we can clone the receiver and spawn a new thread
        }
    }
}
