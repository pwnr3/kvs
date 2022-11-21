use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use kvs::thread_pool::{RayonThreadPool, SharedQueueThreadPool, ThreadPool};
use kvs::{KvStore, KvsEngine, SledKvsEngine};
use rand::prelude::*;
use tempfile::TempDir;

const THREADS: [u32; 1] = [2];
const RNG_RANGE: std::ops::Range<i32> = 0..100;

fn pool_set_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_set_bench");
    group.sample_size(50);
    group.bench_function(BenchmarkId::new("SharedQueueThreadPool + kvs", 0), |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                // error during warmup(Os io: Too many open files)
                // use `ulimit -n 5000` to raise fd number from 256(default)
                let store = KvStore::open(temp_dir.path()).unwrap();
                let pool = SharedQueueThreadPool::new(4).unwrap();
                (store, pool)
            },
            |(store, pool)| {
                for i in 0..100 {
                    let store = store.clone();
                    pool.spawn(move || {
                        store.set(format!("key{}", i), "value".to_string()).unwrap();
                    });
                }
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function(BenchmarkId::new("RayonThreadPool + kvs", 0), |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                // `Sled` is slower than `kvs` and I cannot figure out why, so use `kvs`
                let store = KvStore::open(temp_dir.path()).unwrap();
                let pool = RayonThreadPool::new(4).unwrap();
                (store, pool)
            },
            |(store, pool)| {
                for i in 0..100 {
                    let store = store.clone();
                    pool.spawn(move || {
                        store.set(format!("key{}", i), "value".to_string()).unwrap();
                    });
                }
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn pool_get_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_get_bench");
    //group.measurement_time(std::time::Duration::from_secs(100));
    group.sample_size(10);
    for i in THREADS {
        group.bench_with_input(
            BenchmarkId::new("SharedQueueThreadPool({}) + kvs", i),
            &i,
            |b, i| {
                let temp_dir = TempDir::new().unwrap();
                let store = KvStore::open(temp_dir.path()).unwrap();
                let pool = SharedQueueThreadPool::new(*i).unwrap();
                for key_i in RNG_RANGE {
                    store.set(format!("key{}", key_i), "value".into()).unwrap();
                }
                let mut rng = SmallRng::from_seed([0; 32]);
                b.iter(|| {
                    let store = store.clone();
                    let key = format!("key{}", rng.gen_range(RNG_RANGE));
                    pool.spawn(move || {
                        store.get(key).unwrap();
                    });
                });
            },
        );
    }
    for i in THREADS {
        group.bench_with_input(
            format!("SharedQueueThreadPool({}) + sled", i),
            &i,
            |b, i| {
                let temp_dir = TempDir::new().unwrap();
                let store = SledKvsEngine::open(temp_dir.path()).unwrap();
                let pool = SharedQueueThreadPool::new(*i).unwrap();
                for key_i in RNG_RANGE {
                    store.set(format!("key{}", key_i), "value".into()).unwrap();
                }
                let mut rng = SmallRng::from_seed([0; 32]);
                b.iter(|| {
                    let store = store.clone();
                    let key = format!("key{}", rng.gen_range(RNG_RANGE));
                    pool.spawn(move || {
                        store.get(key).unwrap();
                    });
                });
            },
        );
    }
    for i in THREADS {
        group.bench_with_input(format!("RayonThreadPool({}) + kvs", i), &i, |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let store = KvStore::open(temp_dir.path()).unwrap();
            let pool = RayonThreadPool::new(*i).unwrap();
            for key_i in RNG_RANGE {
                store.set(format!("key{}", key_i), "value".into()).unwrap();
            }
            let mut rng = SmallRng::from_seed([0; 32]);
            b.iter(|| {
                let store = store.clone();
                let key = format!("key{}", rng.gen_range(RNG_RANGE));
                pool.spawn(move || {
                    store.get(key).unwrap();
                });
            });
        });
    }
    for i in THREADS {
        group.bench_with_input(format!("RayonThreadPool({}) + sled", i), &i, |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let store = SledKvsEngine::open(temp_dir.path()).unwrap();
            let pool = RayonThreadPool::new(*i).unwrap();
            for key_i in RNG_RANGE {
                store.set(format!("key{}", key_i), "value".into()).unwrap();
            }
            let mut rng = SmallRng::from_seed([0; 32]);
            b.iter(|| {
                let store = store.clone();
                let key = format!("key{}", rng.gen_range(RNG_RANGE));
                pool.spawn(move || {
                    store.get(key).unwrap();
                });
            });
        });
    }
    group.finish();
}

criterion_group!(benches, pool_get_bench, pool_set_bench);
criterion_main!(benches);
