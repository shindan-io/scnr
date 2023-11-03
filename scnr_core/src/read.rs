use crate::ScanError;
use std::io::{Cursor, Read, Seek};

pub trait ScanReadSeek: ScanRead + Seek {
  fn a_function_only_to_(&self) {}
}
impl<T> ScanReadSeek for T where T: Read + Seek {}

pub trait ScanRead: Read {
  fn a_function_only_to_(&self) {}
}
impl<T> ScanRead for T where T: Read {}

pub enum ScanReader<'r> {
  ReadOnly(&'r mut (dyn ScanRead)),
  ReadSeek(&'r mut (dyn ScanReadSeek)),
}

impl std::fmt::Debug for ScanReader<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::ReadOnly(_) => f.debug_struct("ScanReader::ReadOnly").finish(),
      Self::ReadSeek(_) => f.debug_struct("ScanReader::ReadSeek").finish(),
    }
  }
}

impl<'r> ScanReader<'r> {
  pub fn read_only<T: ScanRead>(value: &'r mut T) -> Self {
    Self::ReadOnly(value)
  }
  pub fn read_seek<T: ScanReadSeek>(value: &'r mut T) -> Self {
    Self::ReadSeek(value)
  }

  pub fn into_seekable(self) -> Result<SeekableScanReader<'r>, ScanError> {
    match self {
      Self::ReadOnly(reader) => {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(SeekableScanReader::ReadOnly(Cursor::new(buf)))
      }
      Self::ReadSeek(r) => Ok(SeekableScanReader::ReadSeek(r)),
    }
  }
}

impl Read for ScanReader<'_> {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    match self {
      Self::ReadOnly(reader) => reader.read(buf),
      Self::ReadSeek(reader) => reader.read(buf),
    }
  }
}

pub enum SeekableScanReader<'r> {
  ReadOnly(Cursor<Vec<u8>>),
  ReadSeek(&'r mut (dyn ScanReadSeek)),
}

impl Read for SeekableScanReader<'_> {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    match self {
      Self::ReadOnly(cursor) => cursor.read(buf),
      Self::ReadSeek(reader) => reader.read(buf),
    }
  }
}

impl Seek for SeekableScanReader<'_> {
  fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
    match self {
      Self::ReadOnly(cursor) => cursor.seek(pos),
      Self::ReadSeek(reader) => reader.seek(pos),
    }
  }
}
