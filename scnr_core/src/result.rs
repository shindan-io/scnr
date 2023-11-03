use crate::{ScanContent, ScanError};
use flume::Receiver;

pub struct ScanResult {
  receiver: Receiver<Result<ScanContent, ScanError>>,
}

impl ScanResult {
  pub(crate) fn new(receiver: Receiver<Result<ScanContent, ScanError>>) -> Self {
    Self { receiver }
  }
}

impl IntoIterator for ScanResult {
  type Item = Result<ScanContent, ScanError>;

  type IntoIter = ScanResultIterator;

  fn into_iter(self) -> Self::IntoIter {
    let iterator = self.receiver.into_iter();
    ScanResultIterator { iterator }
  }
}

pub struct ScanResultIterator {
  iterator: flume::IntoIter<Result<ScanContent, ScanError>>,
}

impl Iterator for ScanResultIterator {
  type Item = Result<ScanContent, ScanError>;

  fn next(&mut self) -> Option<Self::Item> {
    self.iterator.next()
  }
}
