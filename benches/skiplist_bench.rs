use criterion::{criterion_group, criterion_main, Criterion};
use lockfree_skiplist::LockFreeSkipList;

fn benchmark_insert_find(c: &mut Criterion) {
    c.bench_function("skiplist_insert_find", |b| {
        let list = LockFreeSkipList::<i32, i32>::new();
        let mut key = 0;
        b.iter(|| {
            list.insert(key, key);
            list.find(&key);
            key += 1;
        });
    });
}

criterion_group!(benches, benchmark_insert_find);
criterion_main!(benches);
