use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 8] = b"DZIDX001";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub version: u32,
    pub table_fingerprint: String,
    pub column: String,
    pub entries: BTreeMap<String, Vec<u64>>,
}

pub struct PersistentIndex {
    path: PathBuf,
    meta: IndexMetadata,
}

impl PersistentIndex {
    pub fn create(
        path: impl AsRef<Path>,
        table_fingerprint: String,
        column: &str,
        entries: BTreeMap<String, Vec<u64>>,
    ) -> Result<Self> {
        let index = Self {
            path: path.as_ref().to_path_buf(),
            meta: IndexMetadata {
                version: 1,
                table_fingerprint,
                column: column.into(),
                entries,
            },
        };
        index.save()?;
        Ok(index)
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut f = File::open(&path)?;
        let mut magic = [0u8; 8];
        f.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(anyhow!("invalid DreamizDB index magic"));
        }

        let mut len = [0u8; 8];
        f.read_exact(&mut len)?;
        let n = u64::from_le_bytes(len) as usize;
        if n > 128 * 1024 * 1024 {
            return Err(anyhow!("index metadata too large"));
        }

        let mut payload = vec![0u8; n];
        f.read_exact(&mut payload)?;
        let meta: IndexMetadata = serde_json::from_slice(&payload)?;

        if meta.version != 1 {
            return Err(anyhow!("unsupported index version"));
        }
        Ok(Self { path, meta })
    }

    pub fn save(&self) -> Result<()> {
        let payload = serde_json::to_vec(&self.meta)?;
        let tmp = self.path.with_extension("tmp");
        let mut f = File::create(&tmp)?;
        f.write_all(MAGIC)?;
        f.write_all(&(payload.len() as u64).to_le_bytes())?;
        f.write_all(&payload)?;
        f.sync_all()?;
        drop(f);
        fs::rename(tmp, &self.path)?;
        Ok(())
    }

    pub fn lookup(&self, key: &str) -> Option<&Vec<u64>> {
        self.meta.entries.get(key)
    }

    pub fn fingerprint(&self) -> &str {
        &self.meta.table_fingerprint
    }
    pub fn column(&self) -> &str {
        &self.meta.column
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn entry_count(&self) -> usize {
        self.meta.entries.len()
    }
}

pub fn fingerprint_bytes(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}
