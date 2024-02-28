use pyo3::prelude::*;
use scnr::options::CommonArgs;
use scnr_core::ScanError;

// https://pyo3.rs/

mod convertions;
use convertions::*;

#[derive(thiserror::Error, Debug)]
pub enum PyScnrError {
  #[error("Error: {0}")]
  Any(#[from] anyhow::Error),
  #[error("Scan error: {0:?}")]
  ScanError(#[from] ScanError),
}

impl std::convert::From<PyScnrError> for PyErr {
  fn from(err: PyScnrError) -> PyErr {
    pyo3::exceptions::PyTypeError::new_err((err.to_string(),))
  }
}

#[pyfunction]
fn scan() -> Result<PyScanResultIterator, PyScnrError> {
  let scanner = scnr::get_scanner_from_options(&CommonArgs::default())?;
  let result = scanner.scan()?;
  Ok(result.into())
}

/// Scnr module for Python
#[pymodule]
fn py_scnr(_py: Python, m: &PyModule) -> PyResult<()> {
  // m.add_function(wrap_pyfunction!(guess_the_number, m)?)?;
  m.add_function(wrap_pyfunction!(scan, m)?)?;

  Ok(())
}

// #[pyclass]
// struct MyIterator {
//   iter: Box<dyn Iterator<Item = PyObject> + Send>,
// }

// fn make_iterator<'p, I, J>(iter: I, py: Python<'p>) -> MyIterator
// where
//   I: Iterator<Item = J> + Send + 'static,
//   J: IntoPy<PyObject>,
// {
//   // Box<dyn Iterator<Item = PyObject> + Send>
//   MyIterator { iter: Box::new(iter.into_iter().map(|x| x.into_py(py))) }
// }
// #[pymethods]
// impl MyIterator {
//   fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
//     slf
//   }
//   fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyObject> {
//     slf.iter.next()
//   }
// }
