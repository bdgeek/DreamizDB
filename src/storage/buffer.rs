use super::page::{PageStore, PAGE_SIZE};
use anyhow::Result;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub reads: u64,
    pub bytes_read: u64,
}

#[derive(Debug)]
pub struct BufferPool {
    capacity: usize,
    pages: HashMap<u64, Vec<u8>>,
    lru: VecDeque<u64>,
    stats: CacheStats,
}

impl BufferPool {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            capacity,
            pages: HashMap::new(),
            lru: VecDeque::new(),
            stats: CacheStats::default(),
        }
    }

    pub fn get_or_read(&mut self, store: &mut PageStore, page_id: u64) -> Result<Vec<u8>> {
        if let Some(data) = self.pages.get(&page_id).cloned() {
            self.touch(page_id);
            self.stats.hits += 1;
            return Ok(data);
        }

        let data = store.read_page(page_id)?;
        self.stats.misses += 1;
        self.stats.reads += 1;
        self.stats.bytes_read += PAGE_SIZE as u64;

        if self.pages.len() == self.capacity {
            if let Some(old) = self.lru.pop_front() {
                self.pages.remove(&old);
                self.stats.evictions += 1;
            }
        }

        self.pages.insert(page_id, data.clone());
        self.lru.push_back(page_id);
        Ok(data)
    }

    pub fn clear(&mut self) {
        self.pages.clear();
        self.lru.clear();
    }

    pub fn stats(&self) -> CacheStats {
        self.stats
    }
    pub fn reset_stats(&mut self) {
        self.stats = CacheStats::default();
    }
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    fn touch(&mut self, page_id: u64) {
        if let Some(pos) = self.lru.iter().position(|&id| id == page_id) {
            self.lru.remove(pos);
        }
        self.lru.push_back(page_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::page::PageStore;

    #[test]
    fn lru_pool_hits_and_evicts() {
        let path = std::env::temp_dir().join("dreamizdb-buffer-test.db");
        let mut store = PageStore::create(&path).unwrap();
        store.write_page(0, b"a").unwrap();
        store.write_page(1, b"b").unwrap();

        let mut pool = BufferPool::new(1);
        pool.get_or_read(&mut store, 0).unwrap();
        pool.get_or_read(&mut store, 0).unwrap();
        pool.get_or_read(&mut store, 1).unwrap();

        assert_eq!(pool.stats().hits, 1);
        assert_eq!(pool.stats().misses, 2);
        assert_eq!(pool.stats().evictions, 1);

        let _ = std::fs::remove_file(path);
    }
}
