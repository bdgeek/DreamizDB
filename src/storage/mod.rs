pub mod buffer;
pub mod index;
pub mod page;
pub mod persistence;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Hot,
    Warm,
    Cool,
    Cold,
    Archive,
}

pub fn tier_for_heat(heat: f64) -> Tier {
    match heat {
        h if h >= 0.80 => Tier::Hot,
        h if h >= 0.55 => Tier::Warm,
        h if h >= 0.30 => Tier::Cool,
        h if h >= 0.10 => Tier::Cold,
        _ => Tier::Archive,
    }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub id: u64,
    pub country: String,
    pub value: f64,
}

#[derive(Debug, Default)]
pub struct Table {
    records: Vec<Record>,
    country_index: Option<BTreeMap<String, Vec<usize>>>,
}

impl Table {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, record: Record) {
        self.records.push(record);
        self.country_index = None;
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn sequential_scan(&self, country: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.country == country)
            .collect()
    }
    pub fn build_country_index(&mut self) {
        let mut index = BTreeMap::new();
        for (pos, r) in self.records.iter().enumerate() {
            index
                .entry(r.country.clone())
                .or_insert_with(Vec::new)
                .push(pos);
        }
        self.country_index = Some(index);
    }
    pub fn indexed_lookup(&self, country: &str) -> Option<Vec<&Record>> {
        let index = self.country_index.as_ref()?;
        Some(
            index
                .get(country)?
                .iter()
                .map(|&p| &self.records[p])
                .collect(),
        )
    }
    pub fn has_index(&self) -> bool {
        self.country_index.is_some()
    }
    pub fn all_records(&self) -> &[Record] {
        &self.records
    }
}
