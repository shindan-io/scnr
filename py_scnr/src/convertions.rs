use std::path::PathBuf;

use pyo3::prelude::*;
use scnr_core::{
  result::{ScanResult, ScanResultIterator},
  Content, ScanContent,
};

#[pyclass]
pub struct PyScanResultIterator {
  result: ScanResultIterator,
}

impl From<ScanResult> for PyScanResultIterator {
  fn from(result: ScanResult) -> Self {
    Self { result: result.into_iter() }
  }
}

#[pymethods]
impl PyScanResultIterator {
  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf
  }
  fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyObject> {
    slf
      .result
      .next()
      .transpose()
      .ok()
      .flatten()
      .map::<PyScanContent, _>(|c| c.into())
      .map(|c| c.into_py(slf.py()))
  }
}

#[pyclass]
#[derive(Debug)]
pub struct PyScanContent {
  pub rel_path: PathBuf,
  pub content: PyContent,
}

pub enum PyContent {
  Json(String),
  Text(String),
  Bytes(Vec<u8>),
}

impl From<ScanContent> for PyScanContent {
  fn from(content: ScanContent) -> Self {
    Self { rel_path: content.rel_path, content: content.content.into() }
  }
}

impl From<Content> for PyContent {
  fn from(content: Content) -> Self {
    match content {
      Content::Json(s) => Self::Json(s.to_string()),
      Content::Text(s) => Self::Text(s),
      Content::Bytes(b) => Self::Bytes(b),
    }
  }
}

impl std::fmt::Debug for PyContent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Json(v) => f.debug_tuple("Json").field(v).finish(),
      Self::Text(s) => f.debug_tuple("Text").field(s).finish(),
      Self::Bytes(_b) => f.debug_tuple("Bytes").field(&"...binary...").finish(),
    }
  }
}

#[pymethods]
impl PyScanContent {
  fn __str__(&self) -> String {
    format!("{self:?}")
  }
}
