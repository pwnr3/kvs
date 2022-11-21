use crate::{thread_pool::ThreadPool, Result};
use rayon;

/// Wrapper of `rayon::ThreadPool`
pub struct RayonThreadPool(rayon::ThreadPool);

impl ThreadPool for RayonThreadPool {
    fn new(threads: u32) -> Result<Self> {
        Ok(Self(
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads as usize)
                .build()?,
        ))
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.0.install(|| job());
    }
}
