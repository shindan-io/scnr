# Just manual: https://github.com/casey/just

_default:
  @just --list --unsorted

# ==================================================================================================
# ==================================================================================================
o________________INIT_COMMANDS: _default

# Clean the workspace
clean:
  cargo clean

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
  #!/usr/bin/env bash
 source .venv/bin/activate 
  cd py_scnr &&  && python3 -m virtualenv .venv && maturin develop

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

sysdiagfile := "sysdiagnose_2023.10.26_14-40-37+0200_iPhone-OS_iPhone_19H349.tar.gz"

test_sysdiagnose_examples: build_py_dev install
  #!/usr/bin/env bash
  set -e
  source py_scnr/.venv/bin/activate
  
  python3 examples/sysdiagnose/sysdiagnose-sys.py {{sysdiagfile}}
  ./examples/sysdiagnose/sysdiagnose-sys.sh {{sysdiagfile}}

  python3 examples/sysdiagnose/sysdiagnose_wifi_known_networks.py {{sysdiagfile}}

  python3 examples/sysdiagnose/apps.py {{sysdiagfile}}

# ==================================================================================================
# ==================================================================================================
o________________BUILD_COMMANDS: _default

# Installs scnr command line from the current workspace
install:
  cargo install --path scnr


# ==================================================================================================
# ==================================================================================================
o________________DEPS_COMMANDS: _default

# Installs cargo tools
install_cargo_tools:
  cargo install cargo-deny
  cargo install --locked maturin


# Installs python virtual env requirements
install_python_venv:
  cd py_scnr && python3 -m venv .venv
  cd py_scnr && pip install -r requirements.txt
  # cd py_scnr && pip freeze > requirements.txt
  echo "now call ---->" 
  echo "source ./py_scnr/.venv/bin/activate"

# Installs build tools & dependencies
[linux]
install_tooling: install_cargo_tools && install_python_venv
  sudo apt install python3-venv
  sudo apt install python3-pip
  pip install virtualenv

# Installs build tools & dependencies
[macos]
install_tooling: install_cargo_tools && install_python_venv
  brew install python pipx
  pipx ensurepath
  pipx install pip
  pip install virtualenv
