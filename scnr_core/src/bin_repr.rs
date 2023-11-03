use base64::{
  engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
  DecodeError, Engine,
};

#[derive(thiserror::Error, Debug)]
pub enum BinReprError {
  #[error("Base64 encode error: {0}")]
  Base64Decode(#[from] DecodeError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinRepr {
  Base64,
}

impl BinRepr {
  #[must_use]
  pub fn to_string(&self, bytes: &[u8]) -> String {
    match self {
      BinRepr::Base64 => to_base64(bytes),
    }
  }
  pub fn from_str(&self, s: &str) -> Result<Vec<u8>, BinReprError> {
    Ok(match self {
      BinRepr::Base64 => from_base64(s)?,
    })
  }
}

pub const NO_PAD_BUT_CAN_DECODE: GeneralPurposeConfig = GeneralPurposeConfig::new()
  .with_encode_padding(false)
  .with_decode_padding_mode(DecodePaddingMode::Indifferent);

const URL_SAFE_ENGINE: GeneralPurpose = GeneralPurpose::new(&base64::alphabet::URL_SAFE, NO_PAD_BUT_CAN_DECODE);

pub fn to_base64<T: AsRef<[u8]>>(data: T) -> String {
  URL_SAFE_ENGINE.encode(data)
}

pub fn from_base64<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, DecodeError> {
  URL_SAFE_ENGINE.decode(data)
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_case::test_matrix;
  use BinRepr::*;

  #[test]
  fn test_base64() -> anyhow::Result<()> {
    let b64 = Base64.to_string("hello".as_bytes());

    assert_eq!(b64, "aGVsbG8");

    let bytes = Base64.from_str(&b64)?;
    let hello = String::from_utf8(bytes)?;

    assert_eq!(hello, "hello");

    Ok(())
  }

  #[test_matrix(
    ["", "hello", "zeliu<>z,.||kj_(){{}[)c){(m\n\t\0.,()", "診断", "れい"],
    [Base64]
  )]

  fn a_lot_more_tests(input: &str, repr: BinRepr) -> anyhow::Result<()> {
    let b64 = repr.to_string(input.as_bytes());
    let bytes = repr.from_str(&b64)?;
    let output = String::from_utf8(bytes)?;

    assert_eq!(input, &output);

    Ok(())
  }
}
