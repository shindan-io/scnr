# Just manual: https://github.com/casey/just

_default:
  @just --list --unsorted

# ==================================================================================================
# ==================================================================================================
o________________INIT_COMMANDS: _default

# Clean the workspace
clean:
  cargo clean

# ==================================================================================================
# ==================================================================================================
o________________DEV_COMMANDS: _default

# Execute all tests
check:
  cargo check --workspace
  cargo clippy --workspace --all-targets --all-features -- -D clippy::pedantic -A clippy::missing_errors_doc -A clippy::wildcard_imports
  cargo fmt --all -- --check

check_deny:
  cargo deny check

# ==================================================================================================
# ==================================================================================================
o________________TEST_COMMANDS: _default

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


