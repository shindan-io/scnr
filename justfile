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
  cd py-scnr && python3 -m venv .venv
  cd py-scnr && pip install -U pip maturin
  cd py-scnr && pip freeze
  echo "now call ---->" 
  echo "source ./py-scnr/.venv/bin/activate" 

# execute all commands to check workspace health, if this command pass, CI should pass as well
all: clean test check check_deny install

# ==================================================================================================
# ==================================================================================================
o________________DEV_COMMANDS: _default

alias c:= check

# Execute all checks
check:
  cargo check --workspace
  cargo clippy --workspace --all-targets --all-features -- -D clippy::pedantic -A clippy::missing_errors_doc -A clippy::wildcard_imports
  cargo fmt --all -- --check

check_deny:
  cargo deny check

docs:
  cargo doc --workspace --no-deps --open

build_py_dev:
  cd py-scnr && maturin develop

# ==================================================================================================
# ==================================================================================================
o________________TEST_COMMANDS: _default

alias t:= test

# Execute all tests
test:
  cargo test --workspace

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


