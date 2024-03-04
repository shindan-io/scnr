use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CfgProfile {
  #[default]
  Standard,
  Sysdiagnose,
  Nothing,
}

#[allow(clippy::from_over_into)]
impl Into<scnr::options::CfgProfile> for CfgProfile {
  fn into(self) -> scnr::options::CfgProfile {
    #[allow(clippy::enum_glob_use)]
    use scnr::options::CfgProfile::*;
    match self {
      CfgProfile::Standard => Standard,
      CfgProfile::Sysdiagnose => Sysdiagnose,
      CfgProfile::Nothing => Nothing,
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

#[allow(clippy::from_over_into)]
impl Into<scnr::options::Plugin> for Plugin {
  fn into(self) -> scnr::options::Plugin {
    #[allow(clippy::enum_glob_use)]
    use scnr::options::Plugin::*;
    match self {
      Plugin::FileSystem => FileSystem,
      Plugin::Json => Json,
      Plugin::Zip => Zip,
      Plugin::TarGz => TarGz,
      Plugin::TarXz => TarXz,
      Plugin::Text => Text,
      Plugin::Plist => Plist,
      Plugin::Sqlite => Sqlite,
      Plugin::Bin => Bin,
    }
  }
}

#[cfg(test)]
mod tests {
  use scnr::options::{CommonArgs, DEFAULT_INPUT};

  use super::*;

  #[test]
  fn ensure_defaults_are_equal() {
    assert_eq!(scnr::options::CfgProfile::default(), CfgProfile::default().into());
  }

  #[test]
  fn ensure_default_options_are_same() {
    let default_args = CommonArgs::default();

    assert_eq!(default_args.input, DEFAULT_INPUT.to_string(), "If this changes, change the pyfunction signatures");
    assert_eq!(default_args.profile, CfgProfile::default().into());
  }
}
