# quantlib-rs development tasks
# Run `just --list` to see available recipes

default:
    @just --list

# ── Build ──────────────────────────────────────────────

# Build the entire workspace
build:
    cargo build --workspace

# Build in release mode
release:
    cargo build --workspace --release

# ── Test ───────────────────────────────────────────────

# Run all tests (using nextest if available, otherwise cargo test)
test:
    cargo nextest run --workspace 2>/dev/null || cargo test --workspace

# Run tests for a specific crate
test-crate crate:
    cargo nextest run -p {{crate}} 2>/dev/null || cargo test -p {{crate}}

# ── Lint ───────────────────────────────────────────────

# Run clippy on the entire workspace
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Format all code
fmt:
    cargo fmt --all

# Run all quality checks (fmt + clippy + test)
check: fmt-check clippy test

# ── Docs ───────────────────────────────────────────────

# Build documentation
doc:
    cargo doc --workspace --no-deps --document-private-items

# Build and open documentation
doc-open:
    cargo doc --workspace --no-deps --document-private-items --open

# ── Bench ──────────────────────────────────────────────

# Run all benchmarks
bench:
    cargo bench --workspace

# ── Clean ──────────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean
