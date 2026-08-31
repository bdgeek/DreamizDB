use dreamizdb::storage::buffer::BufferPool;
use dreamizdb::storage::page::PageStore;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn repeated_page_access_uses_cache() {
    let path = std::env::temp_dir().join(format!(
        "dreamizdb-cache-{}.db",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut store = PageStore::create(&path).unwrap();
    store.write_page(0, b"hello").unwrap();

    let mut pool = BufferPool::new(2);
    pool.get_or_read(&mut store, 0).unwrap();
    pool.get_or_read(&mut store, 0).unwrap();

    let s = pool.stats();
    assert_eq!(s.misses, 1);
    assert_eq!(s.hits, 1);
    assert_eq!(s.reads, 1);

    std::fs::remove_file(path).unwrap();
}
