use crate::{result::ScanResult, ScanContent, ScanContext, ScanError, ScanPlugin, ScanReader};

pub fn exec_plugin_scan(reader: ScanReader<'_>, plugin: impl ScanPlugin) -> anyhow::Result<Vec<Result<ScanContent, ScanError>>> {
  let (context, receiver) = ScanContext::new_test_context();

  plugin.scan(&context, reader)?;

  let iter = ScanResult::new(receiver);

  drop(context); // allow the receiver to close

  let mut res = vec![];
  for content in iter {
    res.push(content);
  }

  Ok(res)
}
