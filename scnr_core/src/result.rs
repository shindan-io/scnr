use crate::{ScanContent, ScanError};
use flume::Receiver;

pub struct ScanResult {
  receiver: Receiver<Result<ScanContent, ScanError>>,
}

impl ScanResult {
  pub(crate) fn new(receiver: Receiver<Result<ScanContent, ScanError>>) -> Self {
    Self { receiver }
  }

  /// The iterator can be huge, use the function with caution
  #[must_use]
  pub fn to_vec(self) -> Vec<Result<ScanContent, ScanError>> {
    self.into_iter().collect()
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
