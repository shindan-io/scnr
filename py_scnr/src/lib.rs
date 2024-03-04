use pyo3::prelude::*;
use scnr::options::{CommonArgs, DEFAULT_INPUT, DEFAULT_JQ_QUERY};
use scnr_core::ScanError;

// https://pyo3.rs/

mod iterators;
use iterators::*;

mod options;
use options::*;

#[derive(thiserror::Error, Debug)]
pub enum PyScnrError {
  #[error("Error: {0}")]
  Any(#[from] anyhow::Error),
  #[error("Scan error: {0:?}")]
  ScanError(#[from] ScanError),
  #[error("Jq error: {0:?}")]
  JqError(#[from] scnr_core::jq::JqError),
}

impl std::convert::From<PyScnrError> for PyErr {
  fn from(err: PyScnrError) -> PyErr {
    pyo3::exceptions::PyTypeError::new_err((err.to_string(),))
  }
}

fn to_scnr_starter(starter: Vec<Plugin>) -> Vec<scnr::options::Plugin> {
  starter.into_iter().map(Into::into).collect()
}

fn to_scnr_cfg(cfg: Vec<(String, Plugin)>) -> Vec<(String, scnr::options::Plugin)> {
  cfg.into_iter().map(|(pattern, plugin)| (pattern, plugin.into())).collect()
}

fn activate_verbose(verbose: bool) {
  if verbose {
    pretty_env_logger::try_init().ok();
  }
}

#[pyfunction]
#[pyo3(signature = (*, input = DEFAULT_INPUT.to_string(), filter=vec![], starter=vec![], cfg=vec![], profile=CfgProfile::default(), verbose=false))]
fn scan(
  input: String,
  filter: Vec<String>,
  starter: Vec<Plugin>,
  cfg: Vec<(String, Plugin)>,
  profile: CfgProfile,
  verbose: bool,
) -> Result<ScanResultIterator, PyScnrError> {
  activate_verbose(verbose);
  let starter = to_scnr_starter(starter);
  let cfg = to_scnr_cfg(cfg);
  let profile = profile.into();
  let common = CommonArgs { input, filter, starter, cfg, profile };
  let scanner = scnr::get_scanner_from_options(&common)?;
  let result = scanner.scan()?;
  Ok(result.into())
}

#[pyfunction]
#[pyo3(signature = (*, input = DEFAULT_INPUT.to_string(), query = DEFAULT_JQ_QUERY, filter=vec![], starter=vec![], cfg=vec![], profile=CfgProfile::default(), verbose=false))]
fn jq(
  input: String,
  query: &str,
  filter: Vec<String>,
  starter: Vec<Plugin>,
  cfg: Vec<(String, Plugin)>,
  profile: CfgProfile,
  verbose: bool,
) -> Result<JqIterator, PyScnrError> {
  activate_verbose(verbose);
  let starter = to_scnr_starter(starter);
  let cfg = to_scnr_cfg(cfg);
  let profile = profile.into();
  let common = CommonArgs { input, filter, starter, cfg, profile };
  let scanner = scnr::get_scanner_from_options(&common)?;
  let result = scanner.scan()?;
  let iterator = JqIterator::new(result, query)?;
  Ok(iterator)
}

/// Scnr module for Python
#[pymodule]
fn py_scnr(_py: Python, m: &PyModule) -> PyResult<()> {
  // m.add_function(wrap_pyfunction!(guess_the_number, m)?)?;
  m.add_function(wrap_pyfunction!(scan, m)?)?;
  m.add_function(wrap_pyfunction!(jq, m)?)?;

  Ok(())
}
