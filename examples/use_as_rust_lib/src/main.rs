use scnr::options::CommonArgs;

fn main() -> anyhow::Result<()> {
  let args = CommonArgs { input: "../../_samples".into(), ..Default::default() };

  let scanner = scnr::get_scanner_from_options(&args)?;
  for entry in scanner.scan()? {
    match entry {
      Ok(entry) => println!("file: {}, as {}", entry.rel_path.display(), entry.content),
      Err(err) => println!("error: {err}"),
    }
  }

  Ok(())
}
