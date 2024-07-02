// reexport all sqlite so no more need to `use rusqlite` in other crates
pub use rusqlite::*;

/// Table columns informations from the sql command `PRAGMA table_info('<TABLE_NAME>')`
///
///  ───────┬────────────────┬─────────┬─────────┬──────────────┬─────────┐
/// │  cid  │     name       │  type   │ notnull │ `dflt_value` │   pk    │
/// │ int32 │   varchar      │ varchar │ boolean │  varchar     │ boolean │
/// ├───────┼────────────────┼─────────┼─────────┼──────────────┼─────────┤
/// │     0 │ `ID`           │ BIGINT  │ false   │              │ false   │
/// │     1 │ `timestamp`    │ DOUBLE  │ false   │              │ false   │
/// │     2 │ `timeInterval` │ DOUBLE  │ false   │              │ false   │
/// │     3 │ `Energy`       │ BIGINT  │ false   │              │ false   │
/// │     4 │ `NodeID`       │ BIGINT  │ false   │              │ false   │
/// │     5 │ `RootNodeID`   │ BIGINT  │ false   │              │ false   │
///
#[allow(dead_code)]
pub struct TableFieldInfos {
  pub column_id: i32,
  pub name: String,
  pub field_type: String,
  pub not_null: bool,
  pub default_value: Option<String>,
  pub primary_key: bool,
}

pub trait SqliteExt {
  /// Returns the list of table names in the database
  fn get_table_names(&self) -> Result<Vec<String>>;

  /// Returns the list of table field names in a table
  /// /!\ the `table_name` parameter is sensitive to SQL INJECTION ! (won't fix it - be sure what you pass in)
  fn get_columns_infos(&self, table_name: &str) -> Result<Vec<TableFieldInfos>>;
}

impl SqliteExt for Connection {
  fn get_table_names(&self) -> Result<Vec<String>> {
    let mut stmt = self.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;")?;
    let mut table_names = vec![];
    let names_iter = stmt.query_map([], |row| row.get(0))?;
    for name in names_iter {
      table_names.push(name?);
    }
    Ok(table_names)
  }

  fn get_columns_infos(&self, table_name: &str) -> Result<Vec<TableFieldInfos>> {
    let mut stmt = self.prepare(&format!("PRAGMA table_info('{table_name}');"))?;
    let columns_info = stmt.query_map([], |row| {
      let infos = TableFieldInfos {
        column_id: row.get(0)?,
        name: row.get(1)?,
        field_type: row.get(2)?,
        not_null: row.get(3)?,
        default_value: row.get(4)?,
        primary_key: row.get(5)?,
      };
      Ok(infos)
    })?;

    columns_info.into_iter().collect()
  }
}

pub struct DisplayableValue(types::Value);
impl std::fmt::Display for DisplayableValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.0 {
      types::Value::Null => write!(f, "<NULL>"),
      types::Value::Integer(i) => write!(f, "{i}"),
      types::Value::Real(r) => write!(f, "{r}"),
      types::Value::Text(t) => write!(f, "{t}"),
      types::Value::Blob(_b) => write!(f, "[...binary...]"),
    }
  }
}

pub trait IntoDisplayable {
  fn into_displayable(self) -> Option<DisplayableValue>;
}

impl IntoDisplayable for types::Value {
  fn into_displayable(self) -> Option<DisplayableValue> {
    match self {
      types::Value::Null => None,
      _ => Some(DisplayableValue(self)),
    }
  }
}

#[allow(dead_code)]
pub trait GetToString {
  fn get_to_string(&self, idx: usize) -> Result<Option<String>>;
}

impl GetToString for &Row<'_> {
  fn get_to_string(&self, idx: usize) -> Result<Option<String>> {
    Ok(self.get::<_, types::Value>(idx)?.into_displayable().map(|v| v.to_string()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() -> anyhow::Result<()> {
    let conn = Connection::open_in_memory().unwrap();
    let table_names = conn.get_table_names()?;
    assert_eq!(table_names.len(), 0);
    Ok(())
  }
}
