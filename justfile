# Run the TUI (default recipe)
run:
    cargo run

# Build the debug binary
build:
    cargo build

# Fast syntax and type check
check:
    cargo check

# Format the code
fmt:
    cargo fmt

# Lint; warnings are errors
lint:
    cargo clippy --all-targets -- -D warnings
