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

  let picker = profiles::get_plugin_picker(opts.profile, opts.cfg)?;

  let scanner = Scanner::new(opts.input, picker);
  let scanner = config_scanner_filter(scanner, &opts.filter)?;

  match opts.cmd.unwrap_or_default() {
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
        let extract_path = output.join(content.rel_path);

        if let Some(extract_dir) = extract_path.parent() {
          std::fs::create_dir_all(extract_dir)?;

          // tracing::info!("Extracting {extract_path}", extract_path = extract_path.display());
          let mut file = std::fs::File::create(extract_path)?;

          match content.content {
            scnr_core::Content::Json(json) => serde_json::to_writer_pretty(file, &json)?,
            scnr_core::Content::Text(text) => file.write_all(text.as_bytes())?,
            scnr_core::Content::Bytes(bytes) => file.write_all(&bytes)?,
          }
        }
      }
      Err(err) => tracing::error!("{err:?}"),
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use scnr_core::Scanner;
  use std::env;

  use crate::{options::CfgProfile, profiles};

  fn create_scanner(start: impl ToString) -> anyhow::Result<Scanner> {
    Ok(Scanner::new(start, profiles::get_plugin_picker(CfgProfile::Standard, vec![])?))
  }

  #[tokio::test]
  async fn sample_test() -> anyhow::Result<()> {
    pretty_env_logger::try_init().ok();

    let samples = env::var("SAMPLES_DIR")?;

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
