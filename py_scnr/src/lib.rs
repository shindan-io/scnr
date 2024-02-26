use pyo3::prelude::*;
use rand::Rng;
use std::cmp::Ordering;
use std::io;

// code from https://www.maturin.rs/tutorial

#[pyfunction]
fn guess_the_number() {
  println!("Guess the number!");

  let secret_number = rand::thread_rng().gen_range(1..101);

  loop {
    println!("Please input your guess.");

    let mut guess = String::new();

    io::stdin().read_line(&mut guess).expect("Failed to read line");

    let guess: u32 = match guess.trim().parse() {
      Ok(num) => num,
      Err(_) => continue,
    };

    println!("You guessed: {guess}");

    match guess.cmp(&secret_number) {
      Ordering::Less => println!("Too small!"),
      Ordering::Greater => println!("Too big!"),
      Ordering::Equal => {
        println!("You win!");
        break;
      }
    }
  }
}

#[pyfunction]
fn plop() {
  println!("haaaaaaaaaaaaaaaaaa");
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn py_scnr(_py: Python, m: &PyModule) -> PyResult<()> {
  // m.add_function(wrap_pyfunction!(guess_the_number, m)?)?;
  m.add_function(wrap_pyfunction!(plop, m)?)?;

  Ok(())
}