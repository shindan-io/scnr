# Just manual: https://github.com/casey/just

_default:
  @just --list --unsorted

# ==================================================================================================
# ==================================================================================================
o________________INIT_COMMANDS: _default

# Clean the workspace
clean:
  cargo clean

install_python_venv:
  cd py_scnr && python3 -m venv .venv
  cd py_scnr && pip install -U pip maturin
  cd py_scnr && pip install maturin[patchelf]
  cd py_scnr && pip freeze
  echo "now call ---->" 
  echo "source ./py_scnr/.venv/bin/activate"

# execute all commands to check workspace health, if this command pass, CI should pass as well
all: clean test check check_deny install

# ==================================================================================================
# ==================================================================================================
o________________DEV_COMMANDS: _default

alias c:= check

# Execute all checks
check:
  cargo check --workspace --tests
  cargo clippy --workspace --all-targets --all-features -- -D clippy::pedantic -A clippy::missing_errors_doc -A clippy::wildcard_imports
  cargo fmt --all -- --check

check_deny:
  cargo deny check

docs:
  cargo doc --workspace --no-deps --open

build_py_dev:
  cd py_scnr && maturin develop

# ==================================================================================================
# ==================================================================================================
o________________TEST_COMMANDS: _default

alias t:= test

# Execute all tests
test: test_rust test_python

test_rust:
  cargo test --workspace

test_python: build_py_dev
  #!/usr/bin/env bash
  cd py_scnr
  source .venv/bin/activate
  python3 -m unittest tests.py

test_examples: test_examples_lib test_examples_cli test_examples_python

test_examples_lib:
  cd examples/use_as_rust_lib && cargo run -p use_as_rust_lib

# launch and assert all examples, this can fail if you have already install an older / different version of the tool
# If so, run `just install` in order to be sure to have the version of this repository, then run this command again
test_examples_cli:
  # command line examples needs installation
  command -v scnr || cargo install --path scnr
  cd examples/grep_throught_sqlite && ./example.sh

test_examples_python: build_py_dev
  #!/usr/bin/env bash
  set -e
  cd examples/iter_from_python && ./example.sh


# ==================================================================================================
# ==================================================================================================
o________________BUILD_COMMANDS: _default

# Installs scnr command line from the current workspace
install:
  cargo install --path scnr


# ==================================================================================================
# ==================================================================================================
o________________DEPS_COMMANDS: _default

# Installs scnr command line from the current workspace
install_tooling:
  cargo install cargo-deny


