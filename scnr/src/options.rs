use clap::{Args, Parser, Subcommand, ValueEnum};
use std::{error::Error, path::PathBuf};

const DEFAULT_INPUT: &str = ".";

pub(crate) fn get_options() -> Opts {
  Opts::parse()
}

#[derive(Parser, Debug, Clone)]
#[command(name = "scnr", about = "All in one super awesome file scanner")]
pub(crate) struct Opts {
  #[arg(short, long, default_value = DEFAULT_INPUT, help = "Input file or directory to start scanning")]
  pub input: String,

  #[arg(short, long, help = "Included glob patterns")]
  pub filter: Vec<String>,

  #[arg(short, long, default_value_t = CfgProfile::Standard,  help = "Plugins configuration profile to start, can be then overriden with cfg args")]
  pub profile: CfgProfile,

  #[arg(short, long, help = "Verbose output - only opt-in traces")]
  pub verbose: bool,

  #[arg(
    short,
    long,
    help = "Override default settings by allowing named plugins to handle certain files using glob patterns (e.g. --cfg *.json=json --cfg *data*.sql=sqlite --cfg **/do_not_deser.json=bin)",
    value_parser = parse_key_val::<String, Plugin>
  )]
  pub cfg: Vec<(String, Plugin)>,

  #[command(subcommand)]
  pub cmd: Option<Command>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum CfgProfile {
  Standard,
  Sysdiagnose,
}

impl std::fmt::Display for CfgProfile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let dbg = format!("{:?}", self).to_lowercase();
    write!(f, "{}", dbg)
  }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Plugin {
  Json,
  Zip,
  TarGz,
  TarXz,
  Text,
  Plist,
  Sqlite,
}

impl Opts {}

#[derive(Subcommand, Debug, Clone)]
#[command(about = "Commands")]
pub enum Command {
  #[command(about = "Scan and output results to the console (allowing you to grep)")]
  Scan(ScanArgs),
  #[command(about = "Scan and output results to files in an output directory")]
  Extract(ExtractArgs),
}

impl Default for Command {
  fn default() -> Self {
    Command::Scan(ScanArgs {})
  }
}

#[derive(Args, Debug, Clone)]
pub struct ScanArgs {}

#[derive(Args, Debug, Clone)]
pub struct ExtractArgs {
  #[arg(short, long, help = "Output directory to extrat all files")]
  pub output: PathBuf,
  #[arg(short, long, help = "Force extraction even if the output directory is not empty")]
  pub force: bool,
}

// =================================================================================================
// https://github.com/clap-rs/clap/blob/master/examples/typed-derive.rs
// =================================================================================================

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
  T: std::str::FromStr,
  T::Err: Error + Send + Sync + 'static,
  U: std::str::FromStr,
  U::Err: Error + Send + Sync + 'static,
{
  let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
  Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
