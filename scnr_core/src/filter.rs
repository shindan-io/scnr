use glob::{MatchOptions, Pattern};
use std::path::Path;

use crate::ScanError;

#[must_use]
pub fn case_insensitive() -> MatchOptions {
  MatchOptions { case_sensitive: false, ..Default::default() }
}

pub trait ScanFilter: Send + Sync {
  fn should_scan(&self, path: &Path) -> bool;
}

pub struct YesMan;

impl ScanFilter for YesMan {
  fn should_scan(&self, _path: &Path) -> bool {
    true
  }
}

pub struct AlwayDeny;

impl ScanFilter for AlwayDeny {
  fn should_scan(&self, _path: &Path) -> bool {
    false
  }
}

pub struct Glob {
  globs: Vec<Pattern>,
}

impl Glob {
  pub fn new(glob: &str) -> Result<Self, ScanError> {
    Ok(Self { globs: vec![Pattern::new(glob)?] })
  }
  pub fn multi(globs: &[String]) -> Result<Self, ScanError> {
    Ok(Self { globs: globs.iter().map(|glob| Pattern::new(glob)).collect::<Result<Vec<_>, _>>()? })
  }
}

impl ScanFilter for Glob {
  fn should_scan(&self, path: &Path) -> bool {
    self.globs.iter().any(|glob| glob.matches_path_with(path, case_insensitive()))
  }
}
