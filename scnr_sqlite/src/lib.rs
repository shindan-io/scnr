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
  #[tracing::instrument(err)]
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
  // use super::*;

  // #[test]
  // fn it_works() {
  //   let result = add(2, 2);
  //   assert_eq!(result, 4);
  // }
}
