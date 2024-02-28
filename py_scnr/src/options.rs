use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CfgProfile {
  #[default]
  Standard,
  Sysdiagnose,
  Nothing,
}

impl Into<scnr::options::CfgProfile> for CfgProfile {
  fn into(self) -> scnr::options::CfgProfile {
    match self {
      CfgProfile::Standard => scnr::options::CfgProfile::Standard,
      CfgProfile::Sysdiagnose => scnr::options::CfgProfile::Sysdiagnose,
      CfgProfile::Nothing => scnr::options::CfgProfile::Nothing,
    }
  }
}

#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Plugin {
  FileSystem,
  Json,
  Zip,
  TarGz,
  TarXz,
  Text,
  Plist,
  Sqlite,
  Bin,
}

impl Into<scnr::options::Plugin> for Plugin {
  fn into(self) -> scnr::options::Plugin {
    match self {
      Plugin::FileSystem => scnr::options::Plugin::FileSystem,
      Plugin::Json => scnr::options::Plugin::Json,
      Plugin::Zip => scnr::options::Plugin::Zip,
      Plugin::TarGz => scnr::options::Plugin::TarGz,
      Plugin::TarXz => scnr::options::Plugin::TarXz,
      Plugin::Text => scnr::options::Plugin::Text,
      Plugin::Plist => scnr::options::Plugin::Plist,
      Plugin::Sqlite => scnr::options::Plugin::Sqlite,
      Plugin::Bin => scnr::options::Plugin::Bin,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ensure_defaults_are_equal() {
    assert_eq!(scnr::options::CfgProfile::default(), CfgProfile::default().into());
  }
}
