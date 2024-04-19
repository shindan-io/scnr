#![allow(clippy::default_trait_access, clippy::module_name_repetitions, clippy::wildcard_imports)]
#![deny(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use rusqlite::{params, types, Connection, OpenFlags};
use scnr_core::*;
use serde_json::{Map, Number, Value};
use std::io::Write;
use tempfile::NamedTempFile;

mod sqlite_ext;
use sqlite_ext::SqliteExt;

#[derive(Debug)]
pub struct SqlitePlugin;

impl ScanPlugin for SqlitePlugin {
  #[tracing::instrument(level = "debug", err)]
  fn scan(&self, context: &ScanContext, mut reader: ScanReader<'_>) -> ScanPluginResult {
    // todo: could be better with https://crates.io/crates/memfd ?

    let mut tmp_file = NamedTempFile::new()?;
    std::io::copy(&mut reader, &mut tmp_file)?;
    tmp_file.flush()?;

    let conn = Connection::open_with_flags(&tmp_file, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let table_names = conn.get_table_names()?;

    for table_name in table_names {
      let mut sttmt = conn.prepare(&format!("SELECT * FROM '{table_name}'"))?;
      let mut rows = sttmt.query(params![])?;

      let mut big_json: Vec<Value> = vec![];

      let columns = conn.get_columns_infos(&table_name)?;

      while let Some(row) = rows.next()? {
        let mut json = Map::new();
        for (i, column) in columns.iter().map(|c| &c.name).enumerate() {
          let value = row.get::<_, types::Value>(i)?;
          json.insert(column.clone(), sqlite_to_json(value, context.bin_repr));
        }

        big_json.push(Value::Object(json));
      }

      tracing::debug!("Sending json array of {} elements for table {}", big_json.len(), &table_name);
      let json_array = Value::Array(big_json);
      // dbg!(&table_name, &json_array);
      context.send_child_content(Content::Json(json_array), table_name)?;
    }

    drop(tmp_file);

    Ok(())
  }
}

fn sqlite_to_json(sql: types::Value, bin_repr: BinRepr) -> serde_json::Value {
  use serde_json::Value as J;
  match sql {
    types::Value::Null => J::Null,
    types::Value::Integer(i) => J::Number(i.into()),
    types::Value::Real(f) => Number::from_f64(f).map_or(J::Null, Into::into),
    types::Value::Text(s) => J::String(s),
    types::Value::Blob(bytes) => J::String(bin_repr.to_string(&bytes)),
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::*;
  use crate::{
    tests_helpers::{exec_plugin_scan, get_samples_path},
    ScanReader,
  };

  fn get_json_contents(sample_path: &str) -> anyhow::Result<Vec<(PathBuf, serde_json::Value)>> {
    let samples_dir = get_samples_path()?;
    let mut file = std::fs::File::open(format!("{samples_dir}/{sample_path}"))?;
    let results = exec_plugin_scan(ScanReader::read_seek(&mut file), &SqlitePlugin)?;

    let mut json_contents = vec![];
    for result in results {
      let result = result?;
      let rel_path = result.rel_path;
      if let Content::Json(json) = result.content {
        json_contents.push((rel_path, json));
      }
    }

    Ok(json_contents)
  }

  #[test]
  fn test() -> anyhow::Result<()> {
    let jsons = get_json_contents("sakila_country_only.db")?;

    assert_eq!(jsons.len(), 1);

    assert_eq!(jsons[0].0, PathBuf::from("country"));
    assert_eq!(jsons[0].1.as_array().unwrap().len(), 109);

    Ok(())
  }

  #[test]
  fn failing_test() -> anyhow::Result<()> {
    let samples_dir = get_samples_path()?;
    let mut file = std::fs::File::open(format!("{samples_dir}/w.tar.gz"))?;

    let result = exec_plugin_scan(ScanReader::read_seek(&mut file), &SqlitePlugin);
    assert!(result.is_err());

    Ok(())
  }
}
