use scnr::options::CommonArgs;

fn main() -> anyhow::Result<()> {
  let mut args = CommonArgs::default();
  args.input = "../../_samples".into();

  let scanner = scnr::get_scanner_from_options(&args)?;
  for entry in scanner.scan()? {
    match entry {
      Ok(entry) => println!("file: {}, as {}", entry.rel_path.display(), entry.content),
      Err(err) => println!("error: {}", err),
    }
  }

  Ok(())
}
