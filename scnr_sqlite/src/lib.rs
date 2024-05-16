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

const JSON_LIMIT: usize = 5000;

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

      // todo: there is now a bug in scnr extract : it will extract all chunks to the same file, so in the end there will be only the last chunk in the file
      let send_big_json = |json: Vec<Value>, already_sent_this_table: bool| {
        if json.is_empty() && already_sent_this_table {
          return Ok(());
        }
        tracing::debug!("Sending json array of {} elements for table {}", json.len(), &table_name);
        let json_array = Value::Array(json);
        context.send_child_content(Content::Json(json_array), &table_name)?;
        ScanPluginResult::Ok(())
      };

      let columns = conn.get_columns_infos(&table_name)?;

      let mut already_sent_this_table = false;

      while let Some(row) = rows.next()? {
        let mut json = Map::new();
        for (i, column) in columns.iter().map(|c| &c.name).enumerate() {
          let value = row.get::<_, types::Value>(i)?;
          json.insert(column.clone(), sqlite_to_json(value, context.bin_repr));
        }

        big_json.push(Value::Object(json));

        if big_json.len() >= JSON_LIMIT {
          send_big_json(big_json, already_sent_this_table)?;
          big_json = vec![];
          already_sent_this_table = true;
        }
      }

      send_big_json(big_json, already_sent_this_table)?;
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
  fn test_read_all_tables() -> anyhow::Result<()> {
    let jsons = get_json_contents("sakila_full.db")?;

    assert_eq!(jsons.len(), 23);

    let real_table_and_counts = jsons
      .into_iter()
      .map(|(path, json)| (path, json.as_array().unwrap().len()))
      .collect::<Vec<_>>();

    let expected_tables_and_counts = [
      ("actor", 200),
      ("address", 603),
      ("category", 16),
      ("city", 600),
      ("country", 109),
      ("customer", 599),
      ("film", 1000),
      ("film_actor", 5000),
      ("film_actor", 462),
      ("film_category", 1000),
      ("film_text", 0),
      ("inventory", 4581),
      ("language", 6),
      ("payment", 5000),
      ("payment", 5000),
      ("payment", 5000),
      ("payment", 1049),
      ("rental", 5000),
      ("rental", 5000),
      ("rental", 5000),
      ("rental", 1044),
      ("staff", 2),
      ("store", 2),
    ]
    .map(|(p, c)| (PathBuf::from(p), c))
    .into_iter()
    .collect::<Vec<_>>();

    assert_eq!(expected_tables_and_counts, real_table_and_counts);

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
