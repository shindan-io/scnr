use crate::PyScnrError;
use pyo3::prelude::*;
use scnr_core::{
  result::{ScanResult, ScanResultIterator as ScnrScanResultIterator},
  Content as ScnrContent, ScanContent as ScnrScanContent,
};
use std::path::PathBuf;

#[pyclass]
pub struct ScanResultIterator {
  result: ScnrScanResultIterator,
}

impl From<ScanResult> for ScanResultIterator {
  fn from(result: ScanResult) -> Self {
    Self { result: result.into_iter() }
  }
}

#[pymethods]
impl ScanResultIterator {
  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf
  }

  fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyObject> {
    while let Some(c) = slf.result.next() {
      if let Ok(c) = c.map_err(|e| tracing::error!("{e:?}")) {
        let c: ScanContent = c.into();
        return Some(c.into_py(slf.py()));
      }
    }
    None
  }
}

#[pyclass]
pub struct JqIterator {
  iter: JqInnerIterator,
}

type JqInnerIterator = Box<dyn Iterator<Item = serde_json::Value> + Send>;

impl JqIterator {
  pub fn new(result: ScanResult, query: &str) -> Result<Self, PyScnrError> {
    let filter = scnr_core::jq::JqFilter::new(query)?;

    let iter = result
      .into_iter()
      .filter_map(|c| c.map_err(|e| tracing::error!("{e:?}")).ok())
      .filter_map(|c| c.content.json().map(|json| (c.rel_path, json)))
      .flat_map(move |(_path, json)| filter.run(json).map_err(|e| tracing::error!("{e:?}")).unwrap_or_default());

    Ok(Self { iter: Box::new(iter) })
  }
}

#[pymethods]
impl JqIterator {
  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf
  }

  fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<String> {
    slf.iter.next().map(|v| v.to_string())
  }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct ScanContent {
  pub rel_path: PathBuf,
  pub content: Content,
}

#[derive(Clone)]
pub enum Content {
  Json(String),
  Text(String),
  Bytes(Vec<u8>),
}

#[pymethods]
impl ScanContent {
  fn __str__(&self) -> String {
    format!("{} : {:?}", self.rel_path.display(), self.content)
  }

  fn json(&self) -> Option<&str> {
    match &self.content {
      Content::Json(s) => Some(s),
      _ => None,
    }
  }
  fn text(&self) -> Option<&str> {
    match &self.content {
      Content::Text(s) => Some(s),
      _ => None,
    }
  }
  fn bytes(&self) -> Option<&[u8]> {
    match &self.content {
      Content::Bytes(b) => Some(b.as_slice()),
      _ => None,
    }
  }

  fn content_type(&self) -> ContentType {
    match &self.content {
      Content::Json(_) => ContentType::Json,
      Content::Text(_) => ContentType::Text,
      Content::Bytes(_) => ContentType::Bytes,
    }
  }
}

#[pyclass]
pub enum ContentType {
  #[pyo3(name = "JSON")]
  Json,
  #[pyo3(name = "TEXT")]
  Text,
  #[pyo3(name = "BYTES")]
  Bytes,
}

impl From<ScnrScanContent> for ScanContent {
  fn from(content: ScnrScanContent) -> Self {
    Self { rel_path: content.rel_path, content: content.content.into() }
  }
}

impl From<ScnrContent> for Content {
  fn from(content: ScnrContent) -> Self {
    match content {
      ScnrContent::Json(s) => Self::Json(s.to_string()),
      ScnrContent::Text(s) => Self::Text(s),
      ScnrContent::Bytes(b) => Self::Bytes(b),
    }
  }
}

impl std::fmt::Debug for Content {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Json(v) => f.debug_tuple("Json").field(v).finish(),
      Self::Text(s) => f.debug_tuple("Text").field(s).finish(),
      Self::Bytes(_b) => f.debug_tuple("Bytes").field(&"...binary...").finish(),
    }
  }
}
