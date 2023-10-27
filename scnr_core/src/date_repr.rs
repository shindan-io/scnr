use std::time::SystemTime;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

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
      DateRepr::Rfc3339 => Into::<OffsetDateTime>::into(datetime).format(&Rfc3339)?,
    })
  }
  pub fn parse(&self, s: &str) -> Result<SystemTime, DateReprError> {
    Ok(match self {
      DateRepr::Rfc3339 => OffsetDateTime::parse(s, &Rfc3339)?.into(),
    })
  }
}
