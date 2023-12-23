use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use sqlite_hashes::{HashState, NamedDigest};

criterion_group!(benches, all_hash_tests);
criterion_main!(benches);

fn all_hash_tests(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashes");
    hash_test::<md5::Md5>(&mut group);
    hash_test::<sha1::Sha1>(&mut group);
    hash_test::<sha2::Sha224>(&mut group);
    hash_test::<sha2::Sha256>(&mut group);
    hash_test::<sha2::Sha384>(&mut group);
    hash_test::<sha2::Sha512>(&mut group);
    hash_test::<noncrypto_digests::Fnv>(&mut group);
    hash_test::<noncrypto_digests::Xxh32>(&mut group);
    hash_test::<noncrypto_digests::Xxh64>(&mut group);
    hash_test::<noncrypto_digests::Xxh3_64>(&mut group);
    hash_test::<noncrypto_digests::Xxh3_128>(&mut group);
    group.finish();
}

fn hash_test<T: NamedDigest + Clone>(group: &mut BenchmarkGroup<WallTime>) {
    for size in [10, 10 * 1024, 1024 * 1024] {
        let data = gen_data(size);
        group.bench_function(BenchmarkId::new(T::name(), size), |b| {
            b.iter(|| {
                let mut state = HashState::<T>::default();
                state.add_value(data.as_slice());
                state.finalize()
            });
        });
    }
}

fn gen_data(size: usize) -> Vec<u8> {
    let mut byte_data: Vec<u8> = Vec::with_capacity(size);
    for i in 0..size {
        #[allow(clippy::cast_possible_truncation)]
        byte_data.push((i % 256) as u8);
    }
    byte_data
}
