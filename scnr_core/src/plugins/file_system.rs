use super::*;
use std::fs::File;

#[derive(Debug)]
pub struct FileSystemPlugin;

impl ScanPlugin for FileSystemPlugin {
  #[tracing::instrument]
  fn can_start(&self, start_param: &str) -> bool {
    true
  }

  #[tracing::instrument(err)]
  fn start(&self, context: &ScanContext, start_param: &str) -> ScanPluginResult {
    let path = PathBuf::from(start_param);

    if path.is_dir() {
      const ALL_FILES: &str = "**";
      let all_files = globwalk::GlobWalkerBuilder::from_patterns(&path, &[ALL_FILES])
        .build()?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file());

      for file in all_files {
        let relative_path = file.path().strip_prefix(&path)?.to_path_buf();
        let mut reader = File::open(file.path())?;
        context.recurse(relative_path, ScanReader::read_seek(&mut reader))?;
      }
    } else if path.is_file() {
      if let Some(file_name) = path.file_name() {
        let relative_path = PathBuf::from(file_name);
        let mut reader = File::open(&path)?;
        context.recurse(relative_path, ScanReader::read_seek(&mut reader))?;
      }
    }

    Ok(())
  }
}
