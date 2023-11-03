use super::*;

#[derive(Debug)]
pub struct TarGzPlugin;

impl ScanPlugin for TarGzPlugin {
  fn can_recurse(&self) -> bool {
    true
  }

  #[tracing::instrument(skip(reader))]
  fn scan(&self, context: &ScanContext, reader: ScanReader<'_>) -> ScanPluginResult {
    let tar = flate2::read::GzDecoder::new(reader);
    let mut archive = tar::Archive::new(tar);

    for entry in archive.entries()? {
      let mut entry = entry?;
      if entry.header().entry_type() != tar::EntryType::file() {
        continue;
      }
      let path = entry.path()?.to_path_buf();
      context.recurse(path, ScanReader::read_only(&mut entry))?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    tests_helpers::{exec_plugin_scan, get_samples_path},
    ScanReader,
  };

  #[test]
  fn test() -> anyhow::Result<()> {
    let samples_dir = get_samples_path()?;
    let mut file = std::fs::File::open(format!("{samples_dir}/w.tar.gz"))?;

    let results = exec_plugin_scan(ScanReader::read_seek(&mut file), &TarGzPlugin)?;
    assert_eq!(results.len(), 3);

    let mut iter = results.into_iter();

    let result = iter.next().expect("?");
    assert!(matches!(result, Ok(scan) if scan.rel_path.as_os_str() == "w/e.json"));

    let result = iter.next().expect("?");
    assert!(matches!(result, Ok(scan) if scan.rel_path.as_os_str() == "f.yaml"));

    let result = iter.next().expect("?");
    assert!(matches!(result, Ok(scan) if scan.rel_path.as_os_str() == "sakila_master.db"));

    Ok(())
  }

  #[test]
  fn failing_test() -> anyhow::Result<()> {
    let samples_dir = get_samples_path()?;
    let mut file = std::fs::File::open(format!("{samples_dir}/z.zip"))?;

    let result = exec_plugin_scan(ScanReader::read_seek(&mut file), &TarGzPlugin);
    assert!(result.is_err());

    Ok(())
  }
}
