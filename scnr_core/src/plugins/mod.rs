use super::*;
use glob::Pattern;

pub mod bin;
pub mod file_system;
pub mod json;
pub mod last_resort;
pub mod targz;
pub mod tarxz;
pub mod text;
pub mod zip;

pub type PluginsList = Vec<Box<dyn ScanPlugin>>;
pub type PluginsGlobsList = Vec<(Option<Pattern>, Box<dyn ScanPlugin>)>;

pub type ScanPluginResult = Result<(), anyhow::Error>;

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
  fn start(&self, _context: &ScanContext, _start_param: &str) -> ScanPluginResult {
    Err(anyhow::anyhow!("This plugin cannot be used as a start plugin"))
  }

  /// Returns true is this plugin can recurse (And thus should be recurse even if the filter does not allow it)
  /// If you call context.recurse in the scan function, this function should return true !
  fn can_recurse(&self) -> bool {
    false
  }

  /// scan the current context and returns a stream of nodes
  fn scan(&self, _context: &ScanContext, _reader: ScanReader<'_>) -> ScanPluginResult {
    Err(anyhow::anyhow!("This plugin cannot scan other plugin nodes"))
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
  pub fn build_as_this(self) -> DefaultPluginPicker {
    DefaultPluginPicker { plugins: Arc::new(self.plugins) }
  }

  pub fn build_with_defaults(mut self) -> Result<DefaultPluginPicker, ScanError> {
    self.plugins.push((None, Box::new(file_system::FileSystemPlugin)));
    let all = Pattern::new("*")?;
    self.plugins.push((Some(all), Box::new(last_resort::LastResortPlugin)));
    Ok(self.build_as_this())
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

  pub fn push_starter_plugin(mut self, plugin: Box<dyn ScanPlugin>) -> Result<Self, ScanError> {
    self.plugins.push((None, plugin));
    Ok(self)
  }
}
