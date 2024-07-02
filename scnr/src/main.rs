#![allow(clippy::default_trait_access, clippy::module_name_repetitions, clippy::wildcard_imports)]
#![deny(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use scnr_core::{bin_repr, jq, Content, Scanner};
use std::{io::Write, path::PathBuf};

use scnr::options::*;

fn main() -> anyhow::Result<()> {
  let opts = scnr::options::get_options();
  if opts.verbose {
    pretty_env_logger::try_init()?;
  }

  let command = opts.cmd.unwrap_or_default();
  let common_args = command.common();

  let scanner = scnr::get_scanner_from_options(common_args)?;

  match command {
    scnr::options::Command::Scan(args) => scan(scanner, args)?,
    scnr::options::Command::Extract(args) => extract(scanner, args)?,
    scnr::options::Command::Jq(args) => jq(scanner, args)?,
  }

  Ok(())
}

fn print_path(out: &mut impl Write, path: &PathBuf, options: &CommonArgs) -> anyhow::Result<()> {
  if options.print_file_names {
    writeln!(out, "{}", path.display())?;
  }
  Ok(())
}

fn print_content(out: &mut impl Write, content: &Content, options: &CommonArgs) -> anyhow::Result<()> {
  match &content {
    scnr_core::Content::Json(json) => {
      let consume_out = &mut *out;
      if options.pretty_print {
        serde_json::to_writer_pretty(consume_out, &json)?;
      } else {
        serde_json::to_writer(consume_out, &json)?;
      }
    }
    scnr_core::Content::Text(text) => writeln!(out, "{text}")?,
    scnr_core::Content::Bytes(bytes) => writeln!(out, "{}", bin_repr::BinRepr::Base64.to_string(&bytes))?,
  }

  writeln!(out)?;

  Ok(())
}

#[tracing::instrument(skip(scanner), err)]
fn scan(scanner: Scanner, args: ScanArgs) -> anyhow::Result<()> {
  let stdout = std::io::stdout();
  let mut lock = stdout.lock();

  let iter = scanner.scan()?;

  for content in iter {
    match content {
      Ok(content) => {
        print_path(&mut lock, &content.rel_path, &args.common)?;
        print_content(&mut lock, &content.content, &args.common)?;
      }
      Err(err) => tracing::error!("{err:?}"),
    }
  }

  Ok(())
}

#[tracing::instrument(skip(scanner), err)]
fn jq(scanner: Scanner, args: JqArgs) -> anyhow::Result<()> {
  let stdout = std::io::stdout();
  let mut lock = stdout.lock();

  let jq_filter = jq::JqFilter::new(&args.query)?;

  let iter = scanner.scan()?;

  for content in iter {
    match content {
      Ok(content) => {
        if let Some(json) = content.content.json() {
          print_path(&mut lock, &content.rel_path, &args.common)?;
          for element in jq_filter.run(json)? {
            print_content(&mut lock, &Content::Json(element), &args.common)?;
          }
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
  use clap::Parser;
  use scnr::options::Opts;
  use scnr_core::{tests_helpers::get_samples_path, Content, ScanContent, Scanner};

  fn create_scanner(args: &str) -> anyhow::Result<Scanner> {
    let opts = Opts::parse_from(args.split(' '));
    let command = opts.cmd.unwrap_or_default();
    let common_args = command.common();
    let scanner = scnr::get_scanner_from_options(common_args)?;
    Ok(scanner)
  }

  #[test]
  fn sample_to_console() -> anyhow::Result<()> {
    let samples = get_samples_path()?;
    test_scnr_scan_output(&format!("scnr scan -i {samples}"), 24, 7, 4, 2)
  }

  #[test]
  fn sample_to_console_sysdiag_profil() -> anyhow::Result<()> {
    let samples = get_samples_path()?;
    test_scnr_scan_output(&format!("scnr scan -i {samples} -p sysdiagnose"), 39, 7, 1, 3)
  }

  fn test_scnr_scan_output(
    cmd: &str,
    expected_jsons: usize,
    expected_texts: usize,
    expected_bins: usize,
    expected_errs: usize,
  ) -> anyhow::Result<()> {
    pretty_env_logger::try_init().ok();

    let scanner = create_scanner(cmd)?;
    let iter = scanner.scan()?;

    let mut jsons_count = 0;
    let mut texts_count = 0;
    let mut bins_count = 0;
    let mut errs_count = 0;

    for content in iter {
      match content {
        Ok(content) => {
          tracing::info!("{content}");
          match content.content {
            Content::Json(_) => jsons_count += 1,
            Content::Text(_) => texts_count += 1,
            Content::Bytes(_) => bins_count += 1,
          }
        }
        Err(err) => {
          tracing::error!("{err:?}");
          errs_count += 1;
        }
      }
    }

    assert_eq!((jsons_count, texts_count, bins_count, errs_count), (expected_jsons, expected_texts, expected_bins, expected_errs));

    Ok(())
  }

  #[test]
  fn nothing_profile_will_return_nothing() -> anyhow::Result<()> {
    let samples = get_samples_path()?;
    let command_line = format!("scnr scan -i {samples} -p nothing");

    let results = create_scanner(&command_line)?.scan()?.to_vec();
    assert!(results.is_empty());

    Ok(())
  }

  #[test]
  fn get_only_one_file_type() -> anyhow::Result<()> {
    let samples = get_samples_path()?;
    let command_line = format!("scnr scan -i {samples} -p nothing --cfg json.json=json --starter file-system");

    let results = create_scanner(&command_line)?.scan()?.to_vec();
    assert_eq!(results.len(), 1);
    assert!(matches!(&results[0], Ok(ScanContent { rel_path, content: Content::Json(_json) }) if rel_path.as_os_str() == "json.json"));

    Ok(())
  }
}
