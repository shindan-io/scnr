use std::time::SystemTime;
use time::format_description::well_known::Rfc3339 as Rfc3339Format;
use time::OffsetDateTime;

#[derive(thiserror::Error, Debug)]
pub enum DateReprError {
  #[error("Date format error: {0}")]
  FormatError(#[from] time::error::Format),
  #[error("Date parse error: {0}")]
  ParseError(#[from] time::error::Parse),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateRepr {
  Rfc3339,
}

impl DateRepr {
  pub fn to_string(&self, datetime: SystemTime) -> Result<String, DateReprError> {
    Ok(match self {
      DateRepr::Rfc3339 => Into::<OffsetDateTime>::into(datetime).format(&Rfc3339Format)?,
    })
  }
  pub fn parse(&self, s: &str) -> Result<SystemTime, DateReprError> {
    Ok(match self {
      DateRepr::Rfc3339 => OffsetDateTime::parse(s, &Rfc3339Format)?.into(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_case::test_matrix;
  use DateRepr::*;

  #[test]
  fn test_rfc3339() -> anyhow::Result<()> {
    let rfc3339 = Rfc3339.to_string(SystemTime::UNIX_EPOCH)?;

    assert_eq!(rfc3339, "1970-01-01T00:00:00Z");

    let datetime = Rfc3339.parse(&rfc3339)?;

    assert_eq!(datetime, SystemTime::UNIX_EPOCH);

    Ok(())
  }

  #[test_matrix(
    [("1970-01-01T00:00:00Z", "1970-01-01T00:00:00Z"), ("2021-01-01T00:00:00Z", "2021-01-01T00:00:00Z"), ("2021-01-01T00:00:00+02:12", "2020-12-31T21:48:00Z")],
    [Rfc3339]
  )]
  fn test_rfc3339_parse((s, expected): (&str, &str), repr: DateRepr) -> anyhow::Result<()> {
    let datetime = repr.parse(s)?;
    let str_repr = repr.to_string(datetime)?;
    assert_eq!(str_repr, expected);
    Ok(())
  }
}
