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
}
