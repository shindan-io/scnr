use std::{io::Read, path::PathBuf, sync::Arc};

use flume::Sender;

pub mod bin_repr;
pub mod date_repr;
pub mod filter;
pub mod helpers;
pub mod plugins;
pub mod reader;
pub mod result;

pub use bin_repr::BinRepr;
pub use date_repr::DateRepr;
pub use filter::ScanFilter;
use plugins::PluginPicker;
pub use plugins::{ScanPlugin, ScanPluginResult};
pub use reader::ScanReader;

pub enum Content {
  Json(serde_json::Value),
  Text(String),
  Bytes(Vec<u8>),
}

impl std::fmt::Display for Content {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Content::Json(v) => write!(f, "{}", v),
      Content::Text(s) => write!(f, "{}", s),
      Content::Bytes(b) => write!(f, "{}", bin_repr::BinRepr::Base64.to_string(b)),
    }
  }
}

impl std::fmt::Debug for Content {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Json(v) => f.debug_tuple("Json").field(v).finish(),
      Self::Text(s) => f.debug_tuple("Text").field(s).finish(),
      Self::Bytes(b) => f.debug_tuple("Bytes").field(&bin_repr::BinRepr::Base64.to_string(b)).finish(),
    }
  }
}

#[derive(Debug)]
pub struct ScanContent {
  pub rel_path: PathBuf,
  pub content: Content,
}

impl std::fmt::Display for ScanContent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}", self.rel_path.to_string_lossy(), self.content)
  }
}

#[derive(thiserror::Error, Debug)]
pub enum ScanError {
  #[error("Io error: {0}")]
  Io(#[from] std::io::Error),
  #[error("Srip prefix error: {0}")]
  StripPrefixError(#[from] std::path::StripPrefixError),
  #[error("No plugin could scan this path.")]
  NoPluginCouldScan,
  #[error("Cannot open node.")]
  CannotOpenNode,
  #[error("Plugin failed to scan in this context: {0}")]
  PluginFailedToScanInThisContext(&'static str),
  #[error("Unable to send content: {0}")]
  SendError(#[from] flume::SendError<Result<ScanContent, Box<ScanError>>>),
  #[error("Globwalk error: {0}")]
  GlobError(#[from] globwalk::GlobError),
  #[error("Pattern error: {0}")]
  PatternError(#[from] glob::PatternError),
  #[error("Walkdir error: {0}")]
  WalkDirError(#[from] walkdir::Error),
  #[error("Not able to read & seek from this reader")]
  ScanReaderNotSeek,
  #[error(transparent)]
  BinReprError(#[from] bin_repr::BinReprError),
  #[error(transparent)]
  DateReprError(#[from] date_repr::DateReprError),
  #[error(transparent)]
  Any(#[from] anyhow::Error),
}

pub struct Scanner {
  root_start: String,
  filter: Arc<Box<dyn ScanFilter>>,
  plugin_picker: Arc<Box<dyn PluginPicker>>,
}

impl Scanner {
  pub fn new(start: impl ToString, plugin_picker: impl PluginPicker + 'static) -> Self {
    Self {
      root_start: start.to_string(),
      plugin_picker: Arc::new(Box::new(plugin_picker)),
      filter: Arc::new(Box::new(filter::YesManFilter)),
    }
  }

  pub fn with_filter(mut self, filter: impl ScanFilter + 'static) -> Self {
    self.filter = Arc::new(Box::new(filter));
    self
  }

  /// Start a thread and returns a content receiver
  pub fn scan(self) -> Result<result::ScanResult, ScanError> {
    let (sender, receiver) = flume::unbounded::<Result<ScanContent, ScanError>>();

    // scan in a thread
    let _thread = std::thread::spawn(move || {
      let context = ScanContext::new(self.root_start, self.plugin_picker, self.filter, sender);
      if let Err(scan_err) = context.scan() {
        tracing::error!("{scan_err:?}");
      }
    });

    let iter = result::ScanResult::new(receiver);

    Ok(iter)
  }

  // Returns all contents in a vec (use it only for small scan)
  pub fn get(self) -> Result<Vec<ScanContent>, ScanError> {
    let mut res = vec![];
    let iter = self.scan()?;
    for content in iter {
      res.push(content?);
    }
    Ok(res)
  }
}

pub struct ScanContext {
  root_start: Arc<String>,
  rel_path: PathBuf,
  filter: Arc<Box<dyn ScanFilter>>,
  plugin_picker: Arc<Box<dyn PluginPicker>>,
  sender: Sender<Result<ScanContent, ScanError>>,

  /// The binary representation of the data, it's just an helper to convert bytes to string
  pub bin_repr: BinRepr,
  pub date_repr: DateRepr,
}

impl std::fmt::Debug for ScanContext {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ScannerContext").field("full_path", &self.rel_path).finish()
  }
}

impl ScanContext {
  pub fn new(
    start: impl ToString,
    plugin_picker: Arc<Box<dyn PluginPicker>>,
    filter: Arc<Box<dyn ScanFilter>>,
    sender: Sender<Result<ScanContent, ScanError>>,
  ) -> Self {
    Self {
      root_start: Arc::new(start.to_string()),
      rel_path: PathBuf::new(),
      filter,
      plugin_picker,
      sender,
      bin_repr: BinRepr::Base64,
      date_repr: DateRepr::Rfc3339,
    }
  }

  pub fn current_path(&self) -> &PathBuf {
    &self.rel_path
  }

  #[tracing::instrument(err)]
  fn scan(self) -> Result<(), ScanError> {
    if let Some(start_plugin) = self.plugin_picker.pick_start(&self.root_start) {
      start_plugin.start(&self, &self.root_start)?;
      Ok(())
    } else {
      Err(ScanError::NoPluginCouldScan)
    }
  }

  pub fn has_current_extension(&self, extension: &str) -> bool {
    self.rel_path.extension().is_some_and(|x| x.to_ascii_lowercase() == extension)
  }

  pub fn has_current_end_with(&self, extension: &str) -> bool {
    self
      .rel_path
      .file_name()
      .is_some_and(|x| x.to_string_lossy().to_lowercase().contains(extension))
  }

  #[tracing::instrument(skip(reader), err)]
  pub fn recurse<'r>(&self, relative_path: impl Into<PathBuf> + std::fmt::Debug, reader: ScanReader<'r>) -> Result<(), ScanError> {
    let new_path = self.rel_path.join(relative_path.into());

    let child_context = Self {
      root_start: self.root_start.clone(),
      rel_path: new_path,
      filter: self.filter.clone(),
      plugin_picker: self.plugin_picker.clone(),
      sender: self.sender.clone(),
      bin_repr: self.bin_repr,
      date_repr: self.date_repr,
    };

    if let Some(plugin) = self.plugin_picker.pick_scan(&child_context) {
      let plugin_name = plugin.name();
      let display_rel = child_context.rel_path.display();
      if !(plugin.can_recurse() || self.filter.should_scan(&child_context.rel_path)) {
        // tracing::debug!("No recursion on {plugin_name}: {display_rel}.");
        return Ok(());
      }

      tracing::info!("Recurse scan with on {plugin_name}: {display_rel}.");
      if let Err(scan_error) = plugin.scan(&child_context, reader) {
        tracing::error!("{plugin_name} failed to scan `{display_rel}`.");
        self.send(Err(scan_error.into()))?
      }

      return Ok(());
    }

    Ok(())
  }

  #[tracing::instrument(skip(self, content), fields(content = %content), err)]
  pub fn send_content(&self, content: Content) -> Result<(), ScanError> {
    let content = ScanContent { rel_path: self.rel_path.clone(), content };
    self.send(Ok(content))
  }

  #[tracing::instrument(skip(self, content), fields(content = %content), err)]
  pub fn send_child_content(&self, content: Content, child_name: impl Into<PathBuf> + std::fmt::Debug) -> Result<(), ScanError> {
    let child_path = self.rel_path.join(child_name.into());
    let content = ScanContent { rel_path: child_path, content };
    self.send(Ok(content))
  }

  fn send(&self, content: Result<ScanContent, ScanError>) -> Result<(), ScanError> {
    let res = self.sender.send(content);
    if let Err(e) = res {
      let e = e.into_inner();
      tracing::error!("Error while sending content: {e:?}");
      return e.map(|_| ());
    }
    Ok(())
  }
}