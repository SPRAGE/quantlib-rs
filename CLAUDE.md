# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Goal

A complete, idiomatic, 1:1 translation of the [QuantLib C++ library](https://github.com/lballabio/QuantLib) into Rust, preserving every public type, function, algorithm, and behavioral contract. See `plan.md` for the full translation plan and `INSTRUCTIONS.md` for detailed conventions and workflow.

## Commands

```sh
just build              # build entire workspace
just test               # run all tests (nextest if available, else cargo test)
just test-crate ql-time # run tests for a specific crate
just clippy             # clippy with -D warnings (all targets)
just fmt                # auto-format (max_width=100)
just fmt-check          # check formatting
just check              # fmt-check + clippy + test (must pass before committing)
just doc-open           # build and open docs
just bench              # run all benchmarks
```

To run a single test: `cargo test -p ql-time test_name` or `cargo nextest run -p ql-time -E 'test(test_name)'`.

## Architecture

16-crate Cargo workspace. Strict dependency order — no cycles:

```
ql-core  (zero external deps — foundation)
  ↑
  ├── ql-time, ql-math
  ├── ql-currencies, ql-quotes, ql-indexes
  ├── ql-termstructures  (← ql-time, ql-math, ql-quotes)
  │     ├── ql-processes  →  ql-models
  │     ├── ql-cashflows  →  ql-instruments  →  ql-pricingengines
  │     └── ql-methods
  ├── ql-experimental  (depends on everything)
  └── ql-legacy        (ql-core, ql-math, ql-models)

quantlib/  (facade — re-exports all crates)
```

If a type needs to be shared across crates, push it down to `ql-core`. Never introduce circular dependencies.

## Code Conventions

**Every module file** begins with:
```rust
//! `TypeName` — short description (translates `ql/path/to/file.hpp`).
```

**Every `lib.rs`** begins with:
```rust
//! # ql-{name}
//!
//! One-line description.

#![warn(missing_docs)]
#![forbid(unsafe_code)]
```

**Type aliases** (all in `ql-core::lib.rs`) — use these in all signatures:
- `Real = f64`, `Rate`, `Spread`, `Volatility`, `DiscountFactor`, `Price`, `Decimal = f64`
- `Time = f64` (year fractions), `Integer = i32`, `BigInteger = i64`
- `Natural = u32`, `BigNatural = u64`, `Size = usize`

**Error handling** — use `ql_core::Result<T>` and the macros:
```rust
ensure!(condition, "message {var}");  // maps to C++ QL_REQUIRE
fail!("message");                     // maps to C++ QL_FAIL
```

**Traits** require `std::fmt::Debug + Send + Sync`. C++ abstract classes → Rust traits; C++ concrete leaf classes → Rust structs implementing those traits.

**Observer pattern** — uses interior mutability so all methods take `&self`. Feature flag `thread-safe-observers` swaps `Rc`/`RefCell` ↔ `Arc`/`RwLock`.

**Newtype wrappers** — `Date(i32)` (serial number), `Array(DVector<Real>)`, `Matrix(DMatrix<Real>)` — decouples the public API from underlying dependencies.

**Handle/RelinkableHandle** — custom smart pointer wrapping `Rc<RefCell<Option<Arc<T>>>>` that maps QuantLib's relinkable handle semantics exactly.

**LazyObject pattern** — cache computed results via `Cell<>`/`RefCell<>` inside `&self` methods (maps C++ `mutable` members).

## Translation Workflow

1. Find the corresponding C++ header(s) in the QuantLib source.
2. Write the Rust module file with the doc header referencing the `.hpp` path.
3. Port the C++ test file first (test-driven translation).
4. Implement until `just check` passes.
5. Update `plan.md` and `INSTRUCTIONS.md` metrics after each session.

## Quality Gates

All of the following must pass before moving on:
- `just check` (fmt + clippy + test) succeeds with zero warnings
- Clippy warnings are errors (`-D warnings`)
- Code formatted to `max_width = 100`
