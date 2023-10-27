use glob::{MatchOptions, Pattern};
use std::path::PathBuf;

use crate::ScanError;

pub fn case_insensitive() -> MatchOptions {
  MatchOptions { case_sensitive: false, ..Default::default() }
}

pub trait ScanFilter: Send + Sync {
  fn should_scan(&self, path: &PathBuf) -> bool;
}

pub struct YesManFilter;

impl ScanFilter for YesManFilter {
  fn should_scan(&self, _path: &PathBuf) -> bool {
    true
  }
}

pub struct NoManFilter;

impl ScanFilter for NoManFilter {
  fn should_scan(&self, _path: &PathBuf) -> bool {
    false
  }
}

pub struct GlobFilter {
  globs: Vec<Pattern>,
}

impl GlobFilter {
  pub fn new(glob: &str) -> Result<Self, ScanError> {
    Ok(Self { globs: vec![Pattern::new(glob)?] })
  }
  pub fn multi(globs: &[String]) -> Result<Self, ScanError> {
    Ok(Self { globs: globs.iter().map(|glob| Pattern::new(glob)).collect::<Result<Vec<_>, _>>()? })
  }
}

impl ScanFilter for GlobFilter {
  fn should_scan(&self, path: &PathBuf) -> bool {
    self.globs.iter().any(|glob| glob.matches_path_with(path, case_insensitive()))
  }
}
