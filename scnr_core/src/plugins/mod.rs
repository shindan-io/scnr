use self::result::ScanResult;

use super::*;
use glob::Pattern;

pub use all_plugins::*;

pub mod all_plugins {
  #[cfg(feature = "plugin_bin")]
  pub use super::bin;
  #[cfg(feature = "plugin_file_system")]
  pub use super::file_system;
  #[cfg(feature = "plugin_json")]
  pub use super::json;
  #[cfg(feature = "plugin_last_resort")]
  pub use super::last_resort;
  #[cfg(feature = "plugin_targz")]
  pub use super::targz;
  #[cfg(feature = "plugin_tarxz")]
  pub use super::tarxz;
  #[cfg(feature = "plugin_text")]
  pub use super::text;
  #[cfg(feature = "plugin_toml")]
  pub use super::toml;
  #[cfg(feature = "plugin_xml")]
  pub use super::xml;
  #[cfg(feature = "plugin_yaml")]
  pub use super::yaml;
  #[cfg(feature = "plugin_zip")]
  pub use super::zip;
}

#[cfg(feature = "plugin_bin")]
pub mod bin;
#[cfg(feature = "plugin_file_system")]
pub mod file_system;
#[cfg(feature = "plugin_json")]
pub mod json;
#[cfg(feature = "plugin_last_resort")]
pub mod last_resort;
#[cfg(feature = "plugin_targz")]
pub mod targz;
#[cfg(feature = "plugin_tarxz")]
pub mod tarxz;
#[cfg(feature = "plugin_text")]
pub mod text;
#[cfg(feature = "plugin_toml")]
pub mod toml;
#[cfg(feature = "plugin_xml")]
pub mod xml;
#[cfg(feature = "plugin_yaml")]
pub mod yaml;
#[cfg(feature = "plugin_zip")]
pub mod zip;

pub type PluginsList = Vec<Box<dyn ScanPlugin>>;
pub type PluginsGlobsList = Vec<(Option<Pattern>, Box<dyn ScanPlugin>)>;

pub trait ScanPlugin: Sync + Send + std::fmt::Debug {
  // Returns the plugin name using Any::type_name
  fn name(&self) -> &'static str {
    std::any::type_name::<Self>()
  }

  /// If returns true, this plugin is a start plugin and can handle the start param
  fn can_start(&self, _start_param: &str) -> bool {
    false
  }

  /// Starts the stream from a simple string parameter
  fn start<'a>(&self, _context: &ScanContext, _start_param: &str) -> ScanIteratorResult<'a> {
    let result: ScanResult = Err(anyhow::anyhow!("This plugin cannot be used as a start plugin").into());
    Ok(Box::new(std::iter::once(result)))
  }

  /// Returns true is this plugin can recurse (And thus should be recurse even if the filter does not allow it)
  /// If you call context.recurse in the scan function, this function should return true !
  fn can_recurse(&self) -> bool {
    false
  }

  /// scan the current context and returns a stream of nodes
  fn scan<'a>(&self, _context: &ScanContext, _reader: ScanReader<'_>) -> ScanIteratorResult<'a> {
    let result: ScanResult = Err(anyhow::anyhow!("This plugin cannot scan other plugin nodes").into());
    Ok(Box::new(std::iter::once(result)))
  }
}

pub trait PluginPicker: Send + Sync {
  fn pick_start(&self, start_param: &str) -> Option<&dyn ScanPlugin>;
  fn pick_scan(&self, context: &ScanContext) -> Option<&dyn ScanPlugin>;
}

pub struct DefaultPluginPicker {
  plugins: Arc<PluginsGlobsList>,
}

impl DefaultPluginPicker {
  #[must_use]
  pub fn builder() -> DefaultPluginPickerBuilder {
    Default::default()
  }
}

impl PluginPicker for DefaultPluginPicker {
  fn pick_start(&self, start_param: &str) -> Option<&dyn ScanPlugin> {
    self
      .plugins
      .iter()
      .map(|(_, p)| p)
      .find(|p| p.can_start(start_param))
      .map(AsRef::as_ref)
  }

  fn pick_scan(&self, context: &ScanContext) -> Option<&dyn ScanPlugin> {
    for (pattern, plugin) in self.plugins.iter() {
      if let Some(pattern) = pattern {
        if pattern.matches_path_with(&context.rel_path, filter::case_insensitive()) {
          return Some(plugin.as_ref());
        }
      }
    }
    None
  }
}

#[derive(Default)]
pub struct DefaultPluginPickerBuilder {
  plugins: PluginsGlobsList,
}

impl DefaultPluginPickerBuilder {
  #[must_use]
  pub fn builder() -> Self {
    Self { plugins: vec![] }
  }

  #[must_use]
  pub fn build(self) -> DefaultPluginPicker {
    DefaultPluginPicker { plugins: Arc::new(self.plugins) }
  }

  pub fn insert_plugin(self, glob: &str, plugin: impl ScanPlugin + 'static) -> Result<Self, ScanError> {
    self.insert_boxed_plugin(glob, Box::new(plugin))
  }

  pub fn insert_boxed_plugin(mut self, glob: &str, plugin: Box<dyn ScanPlugin>) -> Result<Self, ScanError> {
    let pattern = Pattern::new(glob)?;
    self.plugins.insert(0, (Some(pattern), plugin));
    Ok(self)
  }

  pub fn push_plugin(self, glob: &str, plugin: impl ScanPlugin + 'static) -> Result<Self, ScanError> {
    self.push_boxed_plugin(glob, Box::new(plugin))
  }

  pub fn push_boxed_plugin(mut self, glob: &str, plugin: Box<dyn ScanPlugin>) -> Result<Self, ScanError> {
    let pattern = Pattern::new(glob)?;
    self.plugins.push((Some(pattern), plugin));
    Ok(self)
  }

  pub fn push_starter_plugin(self, plugin: impl ScanPlugin + 'static) -> Result<Self, ScanError> {
    self.push_boxed_starter_plugin(Box::new(plugin))
  }

  pub fn push_boxed_starter_plugin(mut self, plugin: Box<dyn ScanPlugin>) -> Result<Self, ScanError> {
    self.plugins.push((None, plugin));
    Ok(self)
  }
}
