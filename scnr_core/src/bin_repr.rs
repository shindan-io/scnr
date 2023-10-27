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
