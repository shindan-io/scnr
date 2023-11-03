use scnr_core::{bin_repr, filter, Scanner};
use std::io::Write;

mod options;
mod profiles;

use options::*;

fn main() -> anyhow::Result<()> {
  let opts = options::get_options();
  if opts.verbose {
    pretty_env_logger::try_init()?;
  }

  let command = opts.cmd.unwrap_or_default();

  let common_args = match &command {
    Command::Scan(c) => &c.common,
    Command::Extract(c) => &c.common,
  }
  .clone();

  let picker = profiles::get_plugin_picker(common_args.profile, common_args.cfg)?;

  let scanner = Scanner::new(common_args.input, picker);
  let scanner = config_scanner_filter(scanner, &common_args.filter)?;

  match command {
    options::Command::Scan(args) => scan(scanner, args)?,
    options::Command::Extract(args) => extract(scanner, args)?,
  }

  Ok(())
}

fn config_scanner_filter(mut scanner: Scanner, filter: &[String]) -> anyhow::Result<Scanner> {
  if !filter.is_empty() {
    scanner = scanner.with_filter(filter::GlobFilter::multi(filter)?);
  }
  Ok(scanner)
}

#[tracing::instrument(skip(scanner), err)]
fn scan(scanner: Scanner, _args: ScanArgs) -> anyhow::Result<()> {
  let stdout = std::io::stdout();
  let mut lock = stdout.lock();

  let iter = scanner.scan()?;

  for content in iter {
    match content {
      Ok(content) => {
        println!("{}", content.rel_path.display());
        match content.content {
          scnr_core::Content::Json(json) => serde_json::to_writer_pretty(&mut lock, &json)?,
          scnr_core::Content::Text(text) => writeln!(lock, "{text}")?,
          scnr_core::Content::Bytes(bytes) => writeln!(lock, "{}", bin_repr::BinRepr::Base64.to_string(&bytes))?,
        }
      }
      Err(err) => tracing::error!("{err:?}"),
    }
  }

  Ok(())
}

#[tracing::instrument(skip(scanner), err)]
fn extract(scanner: Scanner, args: ExtractArgs) -> anyhow::Result<()> {
  let output = args.output;

  if !args.force && output.exists() && output.is_dir() && output.read_dir()?.next().is_some() {
    return Err(anyhow::anyhow!("Output directory is not empty"));
  }

  std::fs::create_dir_all(&output)?;

  let iter = scanner.scan()?;

  for content in iter {
    match content {
      Ok(content) => {
        let rel_path = content.rel_path;
        let content_type = content.content.to_string();
        let extract_path = output.join(&rel_path);

        tracing::info!(
          "Extracting {rel_path} as {content_type} in {extract_path}",
          rel_path = rel_path.display(),
          extract_path = extract_path.display()
        );

        if let Some(extract_dir) = extract_path.parent() {
          if !extract_dir.exists() {
            tracing::debug!("Creating folder {extract_dir}", extract_dir = extract_dir.display());
            std::fs::create_dir_all(extract_dir)?;
          }
        }

        let mut file = std::fs::File::create(extract_path)?;

        match content.content {
          scnr_core::Content::Json(json) => serde_json::to_writer_pretty(file, &json)?,
          scnr_core::Content::Text(text) => file.write_all(text.as_bytes())?,
          scnr_core::Content::Bytes(bytes) => file.write_all(&bytes)?,
        }
      }
      Err(err) => tracing::error!("{err:?}"),
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::{options::CfgProfile, profiles};
  use scnr_core::{tests_helpers::get_samples_path, Scanner};

  fn create_scanner(start: impl ToString) -> anyhow::Result<Scanner> {
    Ok(Scanner::new(start, profiles::get_plugin_picker(CfgProfile::Standard, vec![])?))
  }

  #[test]
  fn sample_test() -> anyhow::Result<()> {
    pretty_env_logger::try_init().ok();

    let samples = get_samples_path()?;

    let scanner = create_scanner(samples)?;
    let iter = scanner.scan()?;

    for content in iter {
      match content {
        Ok(content) => tracing::info!("{content}"),
        Err(err) => tracing::error!("{err:?}"),
      }
    }

    Ok(())
  }
}
