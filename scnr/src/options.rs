use clap::{Args, Parser, Subcommand, ValueEnum};
use std::{error::Error, path::PathBuf};

pub const DEFAULT_INPUT: &str = ".";
pub const DEFAULT_JQ_QUERY: &str = ".";

#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn get_options() -> Opts {
  Opts::parse()
}

#[derive(Parser, Debug, Clone, Default, PartialEq)]
#[command(name = "scnr", author, version, about, long_about = None,)]
#[command(propagate_version = true)]
pub struct Opts {
  #[arg(short, long, help = "Verbose output - only opt-in traces")]
  pub verbose: bool,

  #[command(subcommand)]
  pub cmd: Option<Command>,
}

#[derive(Debug, Clone, Args, PartialEq)]
pub struct CommonArgs {
  #[arg(short, long, default_value = DEFAULT_INPUT, help = "Input file or directory to start scanning")]
  pub input: String,

  #[arg(short, long, help = "Included glob patterns")]
  pub filter: Vec<String>,

  #[arg(
    short,
    long,
    help = "Adds a starter plugin (one that is not associated with any blog pattern, but will be able to start the recursion, like the file-system plugin)"
  )]
  pub starter: Vec<Plugin>,

  #[arg(
    short,
    long,
    help =
    "Override default settings by allowing named plugins to handle specific files using glob patterns (e.g. --cfg *.json=json --cfg *data*.sql=sqlite --cfg **/do_not_deser.json=bin).\nPlugins are added in the inverse order of the command line, but the more precise glob patterns in the end.",
    value_parser = parse_key_val::<String, Plugin>
  )]
  pub cfg: Vec<(String, Plugin)>,

  #[arg(short, long, default_value_t = CfgProfile::default(),  help = "Plugins configuration profile to start with. Profiles are cfg bundles and can be then overridden by cfg args")]
  pub profile: CfgProfile,
}

impl Default for CommonArgs {
  fn default() -> Self {
    CommonArgs { input: DEFAULT_INPUT.to_string(), filter: vec![], profile: CfgProfile::default(), cfg: vec![], starter: vec![] }
  }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Default)]
pub enum CfgProfile {
  #[default]
  Standard,
  Sysdiagnose,
  Nothing,
}

impl std::fmt::Display for CfgProfile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let dbg = format!("{self:?}").to_lowercase();
    write!(f, "{dbg}")
  }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Plugin {
  FileSystem,
  Json,
  Ips,
  Zip,
  TarGz,
  TarXz,
  Text,
  Plist,
  Sqlite,
  Bin,
}

impl Opts {}

#[derive(Subcommand, Debug, Clone, PartialEq)]
#[command(about = "Commands")]
pub enum Command {
  #[command(about = "Scan and output results to the console (allowing you to grep)")]
  Scan(ScanArgs),
  #[command(about = "Scan and output results to files in an output directory")]
  Extract(ExtractArgs),
  #[command(about = "Scan, execute jq filter on all possible results and output to the console")]
  Jq(JqArgs),
}

impl Default for Command {
  fn default() -> Self {
    Command::Scan(ScanArgs::default())
  }
}

impl Command {
  #[must_use]
  pub fn common(&self) -> &CommonArgs {
    match self {
      Command::Scan(c) => &c.common,
      Command::Extract(c) => &c.common,
      Command::Jq(c) => &c.common,
    }
  }
}

#[derive(Args, Debug, Clone, Default, PartialEq)]
pub struct ScanArgs {
  #[command(flatten)]
  pub common: CommonArgs,
}

#[derive(Args, Debug, Clone, Default, PartialEq)]
pub struct ExtractArgs {
  #[command(flatten)]
  pub common: CommonArgs,

  #[arg(short, long, help = "Output directory to extrat all files")]
  pub output: PathBuf,
  #[arg(long, help = "Force extraction even if the output directory is not empty")]
  pub force: bool,
}

#[derive(Args, Debug, Clone, Default, PartialEq)]
pub struct JqArgs {
  #[command(flatten)]
  pub common: CommonArgs,

  #[arg(long, short, help = "Jq query to apply to all 'json-ed' results")]
  pub query: String,

  #[arg(long, short = 'n', help = "Do NOT pretty print the json output")]
  pub no_pretty_print: bool,
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

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn parse_cmd_0() {
    let cmd = "scnr";
    let opts = Opts::parse_from(cmd.split(' '));
    assert_eq!(opts, Opts::default());
  }

  #[test]
  fn parse_cmd_1() {
    let cmd = "scnr scan";
    let opts = Opts::parse_from(cmd.split(' '));
    assert_eq!(opts.cmd, Some(Command::Scan(ScanArgs::default())));
  }

  #[test]
  fn parse_cmd_2() {
    let cmd =
      "scnr -v extract --output /tmp -f *.json --filter=**/*.xml --force -p sysdiagnose --cfg img.svg=json --cfg *.toml=text -s file-system";
    let opts = Opts::parse_from(cmd.split(' '));
    assert!(opts.verbose);
    assert_eq!(
      opts.cmd,
      Some(Command::Extract(ExtractArgs {
        common: CommonArgs {
          input: DEFAULT_INPUT.to_string(),
          filter: vec!["*.json".into(), "**/*.xml".into()],
          profile: CfgProfile::Sysdiagnose,
          cfg: vec![("img.svg".into(), Plugin::Json), ("*.toml".into(), Plugin::Text)],
          starter: vec![Plugin::FileSystem],
        },
        output: PathBuf::from("/tmp"),
        force: true,
      }))
    );
  }
}
