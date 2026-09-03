use super::buffer::{BufferPool, CacheStats};
use super::index::{fingerprint_bytes, PersistentIndex};
use super::page::{IoStats, PageStore};
use crate::storage::{Record, Table};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::Path;

pub struct PersistentTable {
    store: PageStore,
    index: PersistentIndex,
    buffer: BufferPool,
    page_count: u64,
}

impl PersistentTable {
    pub fn create_from_table(path: impl AsRef<Path>, table: &Table) -> Result<Self> {
        let data_path = path.as_ref().to_path_buf();
        let index_path = data_path.with_extension("idx");

        let mut store = PageStore::create(&data_path)?;
        let mut page_count = 0;
        let mut index_entries: BTreeMap<String, Vec<u64>> = BTreeMap::new();
        let mut fingerprint_input = Vec::new();

        for record in &table.records {
            let encoded = serde_json::to_vec(record)?;
            fingerprint_input.extend_from_slice(&encoded);
            store.write_page(page_count, &encoded)?;
            index_entries
                .entry(record.country.clone())
                .or_default()
                .push(page_count);
            page_count += 1;
        }

        let fp = fingerprint_bytes(&fingerprint_input);
        let index = PersistentIndex::create(index_path, fp, "country", index_entries)?;

        Ok(Self {
            store,
            index,
            buffer: BufferPool::new(64),
            page_count,
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let data_path = path.as_ref().to_path_buf();
        let index_path = data_path.with_extension("idx");

        let mut store = PageStore::open(&data_path)?;
        let page_count = store.page_count()?;
        let index = PersistentIndex::open(&index_path)?;

        if index.column() != "country" {
            return Err(anyhow!("unsupported persistent index column"));
        }

        let database_fingerprint = Self::calculate_fingerprint(&mut store, page_count)?;

        if database_fingerprint != index.fingerprint() {
            return Err(anyhow!("persistent index does not match database contents"));
        }

        Ok(Self {
            page_count,
            store,
            index,
            buffer: BufferPool::new(64),
        })
    }

    fn calculate_fingerprint(store: &mut PageStore, page_count: u64) -> Result<String> {
        let mut fingerprint_input = Vec::new();

        for page_id in 0..page_count {
            let payload = store.read_page(page_id)?;
            fingerprint_input.extend_from_slice(&payload);
        }

        Ok(fingerprint_bytes(&fingerprint_input))
    }

    pub fn sequential_scan(&mut self, country: &str) -> Result<Vec<Record>> {
        let mut result = Vec::new();

        for page_id in 0..self.page_count {
            let payload = self.buffer.get_or_read(&mut self.store, page_id)?;
            let record: Record = serde_json::from_slice(&payload)?;

            if record.country == country {
                result.push(record);
            }
        }

        Ok(result)
    }

    pub fn indexed_lookup(&mut self, country: &str) -> Result<Vec<Record>> {
        let mut result = Vec::new();

        if let Some(page_ids) = self.index.lookup(country) {
            for &page_id in page_ids {
                let payload = self.buffer.get_or_read(&mut self.store, page_id)?;
                result.push(serde_json::from_slice(&payload)?);
            }
        }

        Ok(result)
    }

    pub fn page_count(&self) -> u64 {
        self.page_count
    }

    pub fn io_stats(&self) -> IoStats {
        self.store.stats()
    }

    pub fn cache_stats(&self) -> CacheStats {
        self.buffer.stats()
    }

    pub fn reset_metrics(&mut self) {
        self.store.reset_stats();
        self.buffer.reset_stats();
    }

    pub fn clear_cache(&mut self) {
        self.buffer.clear();
    }

    pub fn index_entry_count(&self) -> usize {
        self.index.entry_count()
    }
}
