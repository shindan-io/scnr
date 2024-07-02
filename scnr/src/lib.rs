use options::CommonArgs;
use scnr_core::{filter::Glob, Scanner, ScannerOptions};

pub mod options;
pub mod profiles;
pub use scnr_core as core;

pub fn get_scanner_from_options(common_args: &CommonArgs) -> Result<Scanner, anyhow::Error> {
  let options = ScannerOptions::default();
  let picker = profiles::get_plugin_picker(common_args.profile, &common_args.cfg, &common_args.starter, &options)?;
  let scanner = Scanner::new(&common_args.input, picker);
  let scanner = config_scanner_filter(scanner, &common_args.filter)?;
  Ok(scanner)
}

pub fn config_scanner_filter(mut scanner: Scanner, filter: &[String]) -> anyhow::Result<Scanner> {
  if !filter.is_empty() {
    scanner = scanner.with_filter(Glob::multi(filter)?);
  }
  Ok(scanner)
}
