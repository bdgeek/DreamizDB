use anyhow::{bail, Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub const PAGE_SIZE: usize = 4096;
const MAGIC: &[u8; 4] = b"DZPG";
const HEADER: usize = 16;
const PAYLOAD: usize = PAGE_SIZE - HEADER;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IoStats {
    pub reads: u64,
    pub writes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
}

#[derive(Debug)]
pub struct PageStore {
    file: File,
    stats: Arc<Mutex<IoStats>>,
}

impl PageStore {
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)?;
        Ok(Self {
            file,
            stats: Arc::new(Mutex::new(IoStats::default())),
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        Ok(Self {
            file,
            stats: Arc::new(Mutex::new(IoStats::default())),
        })
    }

    pub fn page_count(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len() / PAGE_SIZE as u64)
    }

    pub fn write_page(&mut self, page_id: u64, payload: &[u8]) -> Result<()> {
        if payload.len() > PAYLOAD {
            bail!("payload exceeds page capacity");
        }
        let mut page = [0u8; PAGE_SIZE];
        page[0..4].copy_from_slice(MAGIC);
        page[4..12].copy_from_slice(&page_id.to_le_bytes());
        page[12..16].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        page[HEADER..HEADER + payload.len()].copy_from_slice(payload);

        self.file
            .seek(SeekFrom::Start(page_id * PAGE_SIZE as u64))?;
        self.file.write_all(&page)?;
        self.file.flush()?;
        let mut s = self.stats.lock().unwrap();
        s.writes += 1;
        s.bytes_written += PAGE_SIZE as u64;
        Ok(())
    }

    pub fn read_page(&mut self, page_id: u64) -> Result<Vec<u8>> {
        let mut page = [0u8; PAGE_SIZE];
        self.file
            .seek(SeekFrom::Start(page_id * PAGE_SIZE as u64))?;
        self.file
            .read_exact(&mut page)
            .with_context(|| format!("read page {page_id}"))?;

        if &page[0..4] != MAGIC {
            bail!("invalid page magic");
        }
        let stored_id = u64::from_le_bytes(page[4..12].try_into().unwrap());
        if stored_id != page_id {
            bail!("page id mismatch");
        }
        let len = u32::from_le_bytes(page[12..16].try_into().unwrap()) as usize;
        if len > PAYLOAD {
            bail!("invalid page payload length");
        }

        let mut s = self.stats.lock().unwrap();
        s.reads += 1;
        s.bytes_read += PAGE_SIZE as u64;
        Ok(page[HEADER..HEADER + len].to_vec())
    }

    pub fn stats(&self) -> IoStats {
        *self.stats.lock().unwrap()
    }

    pub fn reset_stats(&self) {
        *self.stats.lock().unwrap() = IoStats::default();
    }
}
