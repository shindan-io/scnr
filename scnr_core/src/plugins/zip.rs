use super::*;

#[derive(Debug)]
pub struct ZipPlugin;

impl ScanPlugin for ZipPlugin {
  fn can_recurse(&self) -> bool {
    true
  }

  #[tracing::instrument(skip(reader))]
  fn scan(&self, context: &ScanContext, reader: ScanReader<'_>) -> ScanPluginResult {
    // the BufReader here adapt from not Sized to Sized in order to be able to read
    let mut reader = reader.into_seekable()?;

    let mut zip = ::zip::ZipArchive::new(&mut reader)?;

    for i in 0..zip.len() {
      let mut entry = zip.by_index(i)?;
      if entry.is_dir() {
        continue;
      }
      let file_name = entry.name().to_string();
      let readonly_scan_reader = ScanReader::read_only(&mut entry);
      context.recurse(file_name, readonly_scan_reader)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    plugins::zip::ZipPlugin,
    tests_helpers::{exec_plugin_scan, get_samples_path},
    ScanReader,
  };

  #[test]
  fn test() -> anyhow::Result<()> {
    pretty_env_logger::try_init().ok();
    let samples_dir = get_samples_path()?;
    let mut file = std::fs::File::open(format!("{samples_dir}/z.zip"))?;

    let results = exec_plugin_scan(ScanReader::read_seek(&mut file), &ZipPlugin)?;
    assert_eq!(results.len(), 1);

    let result = results.into_iter().next().expect("?");
    assert!(matches!(result, Ok(scan) if dbg!(scan.rel_path.as_os_str()) == "z/d.txt"));

    Ok(())
  }

  #[test]
  fn failing_test() -> anyhow::Result<()> {
    pretty_env_logger::try_init().ok();
    let samples_dir = get_samples_path()?;
    let mut file = std::fs::File::open(format!("{samples_dir}/w.tar.gz"))?;

    let result = exec_plugin_scan(ScanReader::read_seek(&mut file), &ZipPlugin);
    assert!(result.is_err());

    Ok(())
  }
}
