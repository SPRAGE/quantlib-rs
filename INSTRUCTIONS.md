# quantlib-rs — AI Translation Instructions

> **Purpose:** This document is the single source of truth for any AI assistant
> continuing work on the quantlib-rs project. It captures the project state,
> established conventions, translation methodology, and step-by-step workflow so
> that work can resume from any point without loss of context.
>
> **Last updated:** 2025-02-26

---

## Table of Contents

1. [Project Summary](#1-project-summary)
2. [Current State](#2-current-state)
3. [Architecture & Dependency Graph](#3-architecture--dependency-graph)
4. [Established Code Conventions](#4-established-code-conventions)
5. [C++ → Rust Translation Rules](#5-c--rust-translation-rules)
6. [Workflow: How to Translate a C++ Module](#6-workflow-how-to-translate-a-c-module)
7. [Workflow: How to Port a C++ Test File](#7-workflow-how-to-port-a-c-test-file)
8. [Phase-by-Phase Status & Next Steps](#8-phase-by-phase-status--next-steps)
9. [Quality Gates (must pass before moving on)](#9-quality-gates)
10. [Mandatory: Keeping plan.md and INSTRUCTIONS.md in Sync](#10-mandatory-keeping-planmd-and-instructionsmd-in-sync)
11. [Reference: Key File Locations](#11-reference-key-file-locations)
12. [Reference: External Crate Usage](#12-reference-external-crate-usage)
13. [Common Pitfalls](#13-common-pitfalls)
14. [Work-in-Progress Log](#14-work-in-progress-log)

---

## 1. Project Summary

**Goal:** Produce a complete, idiomatic, 1:1 translation of the QuantLib C++
library into Rust, preserving every public type, function, algorithm, and
behavioral contract.

**Source:** <https://github.com/lballabio/QuantLib> (pin to a specific commit).

**Scale:** ~310,000 LOC of C++ → estimated ~200,000+ LOC of Rust across 16
crates in a Cargo workspace.

**Detailed plan:** See `plan.md` in the workspace root for the full translation
plan, including module mappings, phase descriptions, and risk register.

---

## 2. Current State

### 2.1 High-Level Metrics

| Metric | Value |
|---|---|
| Crates scaffolded | 16/16 (100%) |
| Rust source files | 228 |
| Lines of code | ~48,900 |
| Unit tests (inline) | 845 (all passing) |
| Integration test files (ported from C++ test-suite) | 4 (test_dates, test_calendars, test_day_counters, test_schedule) |
| Integration tests | 59 (all passing) |
| Total tests | 904 (all passing) |
| Build status | ✅ Clean |
| Overall completion | ~15–18% by module coverage |

### 2.2 Per-Crate Status

| Crate | Files | LOC | Tests | % Done | Key Gaps |
|---|---|---|---|---|---|
| `ql-core` | 14 | 1,144 | 31 | ~35% | Missing some utilities |
| `ql-time` | 63 | 12,926 | 342 | ~97% | All schedule tests ported (CDS + non-CDS), GovernmentBond done |
| `ql-math` | 32 | 8,800 | 188 | ~35% | 15 1D + 2 2D interps, 9 solvers, integrals (Simpson/Trapezoid/GaussKronrod/GaussLobatto/TanhSinh/discrete), linear least squares, Brownian bridge, MT19937/Sobol/Halton RNG, Cholesky/SVD/QR/LU/pseudo_sqrt, covariance utilities |
| `ql-currencies` | 11 | 1,054 | 12 | ~70% | Mostly complete |
| `ql-quotes` | 2 | 330 | 9 | ~50% | Missing ~10 quote types |
| `ql-indexes` | 9 | 1,715 | 41 | ~30% | 14 IBOR + 7 overnight + 6 swap + 3 inflation indexes; missing ~30 specific definitions |
| `ql-cashflows` | 7 | 1,601 | 24 | ~30% | Missing CMS, range accrual coupons |
| `ql-processes` | 14 | 2,154 | 58 | ~50% | 12 of ~22 processes |
| `ql-models` | 10 | 1,674 | 30 | ~10% | Missing entire Market Model framework (~160 files) |
| `ql-instruments` | 8 | 1,704 | 27 | ~13% | Missing Cap/Floor, Swaption, CDS, exotics |
| `ql-methods` | 6 | 1,812 | 21 | ~5% | Missing multi-dim FDM, advanced MC/lattice |
| `ql-pricingengines` | 7 | 1,632 | 28 | ~5% | 6 of ~170 engines |
| `ql-termstructures` | 17 | 5,314 | 66 | ~20% | PiecewiseYieldCurve + 4 rate helpers added; missing OIS helpers, VolCurve bootstrap |
| `ql-experimental` | 14 | 4,203 | 33 | ~7% | Missing credit, commodities, ext. FDM |
| `ql-legacy` | 1 | 6 | 0 | 0% | Empty stub |
| `quantlib` (facade) | 1 | 70 | 1 | ✅ | Complete |

---

## 3. Architecture & Dependency Graph

### 3.1 Workspace Layout

```
quantlib-rs/
├── Cargo.toml              # workspace root
├── plan.md                 # detailed translation plan
├── INSTRUCTIONS.md         # this file
├── flake.nix               # Nix dev shell
├── justfile                # task runner (just build, just test, etc.)
├── rustfmt.toml            # max_width=100, edition=2021
├── clippy.toml             # too-many-arguments-threshold=10
├── crates/
│   ├── ql-core/            # types, errors, patterns, utilities
│   ├── ql-time/            # Date, Calendar, DayCounter, Schedule
│   ├── ql-math/            # interpolation, distributions, solvers, RNG, statistics
│   ├── ql-currencies/      # Currency, Money, ExchangeRate
│   ├── ql-quotes/          # Quote trait, SimpleQuote, etc.
│   ├── ql-indexes/         # Index, IBOR, Inflation, Swap indexes
│   ├── ql-termstructures/  # yield, vol, credit, inflation curves
│   ├── ql-processes/       # stochastic processes
│   ├── ql-models/          # calibrated models
│   ├── ql-methods/         # FDM, lattice, Monte Carlo
│   ├── ql-cashflows/       # CashFlow, Coupon, Leg
│   ├── ql-instruments/     # Bond, Swap, Option, etc.
│   ├── ql-pricingengines/  # pricing engine implementations
│   ├── ql-experimental/    # experimental features
│   └── ql-legacy/          # deprecated LIBOR Market Model
└── quantlib/               # facade crate: re-exports everything
```

### 3.2 Dependency Graph

```
ql-core  (zero deps — the foundation)
  ↑
  ├── ql-time       (ql-core)
  ├── ql-math       (ql-core, nalgebra, statrs, rand)
  │
  ├── ql-currencies (ql-core, ql-time)
  ├── ql-quotes     (ql-core)
  ├── ql-indexes    (ql-core, ql-time, ql-currencies)
  │
  ├── ql-termstructures  (ql-core, ql-time, ql-math, ql-quotes)
  │     ↑
  │     ├── ql-processes  (+ ql-termstructures)
  │     │     ↑
  │     │     └── ql-models  (+ ql-processes)
  │     │
  │     ├── ql-cashflows  (+ ql-indexes, ql-termstructures)
  │     │     ↑
  │     │     └── ql-instruments  (+ ql-cashflows)
  │     │           ↑
  │     │           └── ql-pricingengines  (everything above)
  │     │
  │     └── ql-methods  (+ ql-processes)
  │
  ├── ql-experimental  (depends on everything)
  └── ql-legacy        (ql-core, ql-math, ql-models)

quantlib (facade) → re-exports all of the above
```

**Rule:** Never introduce circular dependencies. If crate A depends on crate B,
B must never depend on A. If you need shared types, push them down to `ql-core`.

---

## 4. Established Code Conventions

These conventions are already in use across the codebase. **Follow them exactly.**

### 4.1 Crate-Level

Every `lib.rs` starts with:
```rust
//! # ql-{name}
//!
//! One-line description.

#![warn(missing_docs)]
#![forbid(unsafe_code)]
```

Every `Cargo.toml` uses workspace inheritance:
```toml
[package]
name = "ql-{name}"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "..."

[dependencies]
ql-core.workspace = true     # always present
# ... other workspace deps

[dev-dependencies]
approx.workspace = true
```

### 4.2 Module Structure

- One module file per C++ header/source pair.
- `pub mod` declarations in `lib.rs` with a `///` doc comment on each.
- Convenience re-exports at the bottom of `lib.rs`.
- Submodules use a directory with `mod.rs` (e.g., `interpolations/mod.rs`).

```rust
// lib.rs pattern
/// Human-readable description.
pub mod foo_bar;

// re-export at bottom
pub use foo_bar::FooBar;
```

### 4.3 Module File Header

Every `.rs` file starts with a module-level doc comment referencing the C++ origin:

```rust
//! `FooBar` — short description (translates `ql/path/to/file.hpp`).
```

### 4.4 Type Aliases

All primitive type aliases live in `ql-core/src/lib.rs`:

```rust
pub type Real = f64;
pub type Rate = f64;
pub type Time = f64;
pub type DiscountFactor = f64;
pub type Spread = f64;
pub type Volatility = f64;
pub type Integer = i32;
pub type BigInteger = i64;
pub type Natural = u32;
pub type BigNatural = u64;
pub type Size = usize;
pub type Price = f64;
pub type Decimal = f64;
```

Use these in all function signatures. Import from `ql_core::Real`, etc.

### 4.5 Error Handling

- Single error enum `ql_core::Error` with `thiserror`.
- Variants: `Runtime`, `Precondition`, `Postcondition`, `NullValue`, `Date`, `IndexOutOfRange`.
- `ql_core::Result<T>` is `std::result::Result<T, ql_core::Error>`.
- Use `ql_core::ensure!(condition, "message")` for preconditions (maps to `QL_REQUIRE`).
- Use `ql_core::fail!("message")` for unconditional errors (maps to `QL_FAIL`).

```rust
use ql_core::{ensure, errors::Result};

pub fn foo(x: Real) -> Result<Real> {
    ensure!(x > 0.0, "x must be positive, got {x}");
    Ok(x.sqrt())
}
```

### 4.6 Trait Design

Traits require `std::fmt::Debug + Send + Sync`:

```rust
pub trait Calendar: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &str;
    fn is_business_day(&self, date: Date) -> bool;
    // default methods for adjust, advance, etc.
}
```

Where QuantLib has a class hierarchy, interior (non-leaf) classes become traits
and leaf classes become structs:

```
C++: Instrument → Option → OneAssetOption → VanillaOption
Rust: trait Instrument, trait OptionInstrument: Instrument,
      struct VanillaOption — impl Instrument + OptionInstrument
```

### 4.7 Newtype Wrappers

Wrap external types to maintain a stable public API:

```rust
// ql-math: Array wraps nalgebra::DVector
#[derive(Debug, Clone, PartialEq)]
pub struct Array(DVector<Real>);

// ql-time: Date wraps i32 serial number
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Date(i32);
```

Provide `From`/`Into` conversions for interop.

### 4.8 Observer/Observable Pattern

Uses interior mutability (`RefCell` / `Mutex`) so all methods take `&self`:

```rust
pub trait Observable {
    fn register_observer(&self, observer: Weak<dyn Observer>);
    fn unregister_observer(&self, observer: &Weak<dyn Observer>);
    fn notify_observers(&self);
}

pub trait Observer: Send + Sync {
    fn update(&self);
}
```

Embed `ObservableImpl` in any observable struct. Use `Weak` for back-pointers
to avoid cycles.

### 4.9 Pricing Engine Pattern

```rust
pub trait PricingEngine<Args>: std::fmt::Debug + Send + Sync {
    fn calculate(&self, args: &Args) -> Result<PricingResults>;
}

pub struct PricingResults {
    pub npv: Real,
    pub error_estimate: Option<Real>,
    pub additional_results: HashMap<String, Real>,
}
```

Engines hold `Arc<Process>` or `Arc<dyn YieldTermStructure>`.

### 4.10 Test Conventions

- **Inline unit tests** in `#[cfg(test)] mod tests { ... }` at the bottom of each file.
- **Integration tests** in `crates/ql-*/tests/test_*.rs` — ported from C++ test-suite.
- Use `approx` crate: `assert_abs_diff_eq!(a, b, epsilon = 1e-10)`.
- Use the **same tolerance as the C++ test** (grep for `tolerance` or `eps`).
- Test function names: `#[test] fn test_descriptive_name()`.

### 4.11 Naming Conventions

| Aspect | Convention | Example |
|---|---|---|
| Crate | `ql-{module}` | `ql-core`, `ql-time` |
| File | `snake_case.rs` | `piecewise_yield_curve.rs` |
| Struct | `PascalCase` | `PiecewiseYieldCurve` |
| Trait | `PascalCase` | `YieldTermStructure` |
| Function | `snake_case` | `year_fraction()` |
| Constant | `SCREAMING_SNAKE` | `SQRT_2` |
| Feature flag | `kebab-case` | `thread-safe-observers` |
| Test function | `test_snake_case` | `test_flat_forward_zero_rate` |
| C++ `camelCase` files | → `snake_case` | `daycounter.hpp` → `day_counter.rs` |

### 4.12 Formatting & Linting

- `rustfmt.toml`: `max_width = 100`, `edition = "2021"`, `use_field_init_shorthand = true`, `use_try_shorthand = true`.
- `clippy.toml`: `too-many-arguments-threshold = 10`, `type-complexity-threshold = 300`.
- All code must pass `cargo clippy --workspace -- -D warnings`.
- All code must pass `cargo fmt --all -- --check`.

---

## 5. C++ → Rust Translation Rules

Apply these rules mechanically for every C++ construct encountered.

### 5.1 Types

| C++ | Rust |
|---|---|
| `Real` / `double` | `Real` (= `f64`) |
| `Integer` / `int` | `Integer` (= `i32`) |
| `Size` / `size_t` | `Size` (= `usize`) |
| `shared_ptr<T>` (widely shared) | `Arc<T>` |
| `shared_ptr<T>` (single thread) | `Rc<T>` |
| `shared_ptr<T>` + mutation | `Arc<RwLock<T>>` or `Rc<RefCell<T>>` |
| `unique_ptr<T>` | `Box<T>` |
| `optional<T>` | `Option<T>` |
| `Null<T>()` | `Option<T>` with `None` |
| `vector<T>` | `Vec<T>` |
| `map<K,V>` | `BTreeMap<K,V>` (ordered) or `HashMap<K,V>` |
| `pair<A,B>` | `(A, B)` |
| `string` | `String` |
| `const&` parameter | `&T` |
| Pass by value | Move (default) or `Clone` |

### 5.2 Classes

| C++ Pattern | Rust |
|---|---|
| Abstract base class (pure virtual) | `trait` |
| Concrete class | `struct` + `impl Trait` |
| Multiple inheritance | Multiple trait bounds (`trait A: B + C`) |
| Pimpl / Bridge (e.g., `Calendar`) | `enum` wrapping variants, or `Box<dyn Impl>` |
| `mutable` fields + `const` methods | `Cell<T>` / `RefCell<T>` with `&self` methods |
| Virtual dispatch | `dyn Trait` (trait object) or enum |
| CRTP | Generics + associated types |
| Template class | `struct<T: Bound>` |
| Template specialization | Separate `impl` blocks |
| `friend` function | Public free function in same module |
| Nested class | Nested struct or separate file |
| Copy constructor | `impl Clone` (+ `Copy` for small types) |
| Destructor with side effects | `impl Drop` |

### 5.3 Control Flow

| C++ | Rust |
|---|---|
| `QL_REQUIRE(cond, msg)` | `ql_core::ensure!(cond, msg)` → returns `Err` |
| `QL_ENSURE(cond, msg)` | `ql_core::ensure_post!(cond, msg)` → returns `Err` |
| `QL_FAIL(msg)` | `ql_core::fail!(msg)` → returns `Err` |
| Exception (`throw`) | `return Err(...)` |
| `try { ... } catch { ... }` | `match result { Ok(v) => ..., Err(e) => ... }` |

### 5.4 Operators

| C++ | Rust |
|---|---|
| `operator<<` (ostream) | `impl std::fmt::Display` |
| `operator==`, `!=` | `impl PartialEq` (derive when possible) |
| `operator<`, `<=` | `impl PartialOrd` / `Ord` |
| `operator+`, `-`, `*`, `/` | `impl Add, Sub, Mul, Div` (from `std::ops`) |
| `operator[]` (read) | `impl Index` |
| `operator[]` (write) | `impl IndexMut` |

### 5.5 Other Patterns

| C++ | Rust |
|---|---|
| Namespace | Module |
| `#define` constant | `const` or `const fn` |
| `typedef` / `using` | `type Alias = ...;` |
| `enum` / `enum class` | `#[derive(Debug, Clone, Copy, PartialEq, Eq)] enum` |
| `static` local (lazy init) | `std::sync::LazyLock` or `thread_local!` |
| `#ifdef` feature toggle | `#[cfg(feature = "...")]` |
| Header guards | Not needed (automatic in Rust) |

---

## 6. Workflow: How to Translate a C++ Module

Follow this exact sequence for each C++ file being translated.

### Step 1: Identify the C++ Source

Find the `.hpp` and `.cpp` file pair in the QuantLib source tree. Note:
- Class name(s) and their inheritance
- Public methods and their signatures
- Private/protected members (these become struct fields)
- Template parameters
- Dependencies (`#include` statements → which Rust crate/module to import)

### Step 2: Determine the Target Location

Use the mapping convention:
```
ql/time/calendars/united_states.hpp → crates/ql-time/src/calendars/united_states.rs
ql/math/interpolations/linear_interpolation.hpp → crates/ql-math/src/interpolations/linear.rs
ql/pricingengines/vanilla/analyticeuropeanengine.hpp → crates/ql-pricingengines/src/analytic_european_engine.rs
```

### Step 3: Write the Module File

```rust
//! `TypeName` — description (translates `ql/path/to/file.hpp`).

use ql_core::{ensure, errors::Result, Real};
// ... other imports

/// Doc comment describing the type.
///
/// Corresponds to `QuantLib::TypeName`.
#[derive(Debug, Clone)]
pub struct TypeName {
    // fields matching C++ data members
}

impl TypeName {
    /// Constructor.
    pub fn new(/* params */) -> Result<Self> {
        ensure!(/* preconditions */);
        Ok(Self { /* fields */ })
    }

    // public methods matching C++ public interface
}

// If it implements a trait:
impl SomeTrait for TypeName {
    // ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_construction() {
        // ...
    }
}
```

### Step 4: Register the Module

Add to the crate's `lib.rs`:
```rust
/// Description.
pub mod type_name;

// at bottom, if it's a commonly used type:
pub use type_name::TypeName;
```

### Step 5: Add to Parent `mod.rs` (if in a subdirectory)

```rust
pub mod type_name;
pub use type_name::TypeName;
```

### Step 6: Build and Test

```bash
cargo build -p ql-{crate}
cargo test -p ql-{crate}
cargo clippy -p ql-{crate} -- -D warnings
```

### Step 7: Verify the Facade

If this is a new public type users should access, ensure it's reachable through
the `quantlib` facade crate.

### Step 8: Update `plan.md` and `INSTRUCTIONS.md`

**This step is mandatory.** After every module translation:

1. **Read `plan.md`** — Open and review the relevant phase section (§7–§18)
   and the progress snapshot (§6.1).
2. **Update checkboxes** — Check off completed items in the §23 Next Steps
   task lists. Add new items if the work revealed additional tasks.
3. **Update the per-crate table in §6.1** — Increment the file count, LOC,
   test count, and completeness percentage for the affected crate.
4. **Update `INSTRUCTIONS.md` §8** — Move completed items from "Remaining"
   into "Done" in the corresponding phase section.
5. **Update `INSTRUCTIONS.md` §2.2** — Refresh the per-crate status table
   (files, LOC, tests, % done, key gaps).

This ensures the next session (human or AI) sees accurate state without
having to re-audit the codebase.

---

## 7. Workflow: How to Port a C++ Test File

The C++ test-suite lives in `test-suite/*.cpp` in the QuantLib repo. Port them
as Rust integration tests.

### Step 1: Read the C++ Test File

Identify each `BOOST_AUTO_TEST_CASE` or test function. Note:
- Setup/teardown (often `SavedSettings` to reset global state)
- Numerical tolerances used
- Data tables (hard-coded test vectors)
- Which QuantLib types/functions are exercised

### Step 2: Create the Rust Integration Test File

```
test-suite/dates.cpp → crates/ql-time/tests/test_dates.rs
test-suite/calendars.cpp → crates/ql-time/tests/test_calendars.rs
```

### Step 3: Write the Test

```rust
//! Tests ported from QuantLib test-suite/dates.cpp

use ql_time::*;
use approx::assert_abs_diff_eq;

#[test]
fn test_consistency() {
    // Port the exact logic from testConsistency() in dates.cpp
    // Use the same tolerance as the C++ test
}
```

### Step 4: Handle Common Test Patterns

| C++ Pattern | Rust Equivalent |
|---|---|
| `SavedSettings backup;` | `let _guard = ql_core::ScopedEvaluationDate::new(date);` |
| `BOOST_CHECK_CLOSE(a, b, tol)` | `assert_abs_diff_eq!(a, b, epsilon = tol)` |
| `BOOST_CHECK_EQUAL(a, b)` | `assert_eq!(a, b)` |
| `BOOST_CHECK(cond)` | `assert!(cond)` |
| `BOOST_CHECK_THROW(expr, ExcType)` | `assert!(expr.is_err())` |
| `BOOST_TEST_MESSAGE(msg)` | `eprintln!("{msg}")` or just remove |

### Step 5: Run

```bash
cargo test -p ql-{crate} --test test_name
```

### Step 6: Update `plan.md` and `INSTRUCTIONS.md`

**This step is mandatory.** After every test file is ported:

1. **Update `plan.md` §19** (Test File Mapping) — Mark the test file as ported.
2. **Update `plan.md` §6.1** — Increment the integration test count.
3. **Update `plan.md` §23** — Check off the corresponding task.
4. **Update `INSTRUCTIONS.md` §2.1** — Refresh the integration test count.
5. **Update `INSTRUCTIONS.md` §8** — Move the test from "Remaining" to "Done"
   in the relevant phase.

---

## 8. Phase-by-Phase Status & Next Steps

Phases are ordered by dependency. Complete earlier phases before later ones.
Within a phase, prioritize: **(1) port tests → (2) implement types → (3) pass tests.**

### 🟡 Phase 0 — Scaffolding (~70%)

**Done:** Workspace, all crate stubs, justfile, .gitignore, LICENSE, README, rustfmt, clippy.
**Remaining:**
- [ ] GitHub Actions CI (`cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`)
- [ ] `cargo-llvm-cov` coverage setup
- [ ] Clone QuantLib reference at pinned commit as `reference/` submodule

### 🟡 Phase 1 — Foundation / `ql-core` (~35%)

**Done:** Type aliases, Error enum + macros, Observable/Observer, Handle/RelinkableHandle, LazyObject, Settings, TimeSeries, Visitor, position, compounding, utilities (null, clone, formatters, parsers).
**Remaining:**
- [ ] Port `test-suite/observable.cpp` → integration test
- [ ] Port `test-suite/errors.cpp` → integration test
- [ ] Audit for missing utilities from `ql/utilities/`

### 🟡 Phase 2 — Time & Calendar / `ql-time` (~90%)

**Done:** Date (serial number, MAX bug fixed), Period, Month, Weekday, TimeUnit, Frequency, BusinessDayConvention, Calendar trait, **45/45 country/exchange calendars** (including Thailand added 2025-02-26), JointCalendar, BespokeCalendar, WeekendsOnly, NullCalendar, DayCounter (15+ conventions, Thirty360 BondBasis + ActualActualIsda bugs fixed), Schedule/ScheduleBuilder, DateGeneration, IMM, ASX, ECB, InterestRate. **3 integration test files** ported from C++ test-suite: test_dates.rs (13 tests), test_calendars.rs (16 tests), test_day_counters.rs (13 tests).
**Remaining:**
- [ ] Port `test-suite/schedule.cpp` → `crates/ql-time/tests/test_schedule.rs` (in progress — C++ source fetched, ScheduleBuilder API reviewed)
- [ ] Potential EOM-aware schedule generation refinements (some C++ tests may require improvements to ScheduleBuilder::build)
- [ ] `cds_maturity()` free function (needed for CDS schedule tests)
- [ ] `Schedule::until()` and `Schedule::after()` truncation methods (needed for truncation tests)

### 🟡 Phase 3 — Math / `ql-math` (~35%)

**Done:** Array (nalgebra wrapper), Matrix + matrix utilities, 9 interpolation schemes (Linear, LogLinear, Flat, ForwardFlat, CubicNatural, Lagrange, Akima, MonotoneCubic, SABR), 7 distributions (Normal, Beta, Binomial, ChiSquare, Gamma, Poisson, StudentT), 6 solvers (Brent, Newton, Secant, Bisection, FalsePosition, Ridder), optimizers (Simplex, LevenbergMarquardt, ConjugateGradient, SteepestDescent, BFGS), RNG (Mersenne Twister, Sobol), statistics (General, Incremental, Sequence, Convergence), copulas (Gaussian, Clayton, Frank, Gumbel, StudentT), ODE (Adaptive RK4), integrals (Trapezoid, Simpson, GaussLobatto, GaussKronrod, Gauss quadratures).
**Remaining:**
- [ ] 15 more interpolation schemes (Chebyshev, ConvexMonotone, Parabolic, FritschButland, Kruger, MixedLinear, etc.)
- [ ] Halton quasi-random sequence generator
- [ ] DifferentialEvolution, SimulatedAnnealing, ParticleSwarmOptimization
- [ ] InverseCumulative distribution wrappers
- [ ] SVD, QR decomposition, eigenvalue wrappers (nalgebra-backed)
- [ ] Pseudo-sqrt, BiCGstab
- [ ] Port 10 C++ test-suite files as integration tests

### 🟡 Phase 4 — Financial Primitives (~34%)

**Done:** Currency, Money, ExchangeRate, ExchangeRateManager, 6 regional currency modules, Quote trait + 8 implementations, Index/InterestRateIndex/IborIndex/OvernightIndex/SwapIndex traits, generic IBOR/overnight/inflation index factories.
**Remaining:**
- [ ] ~10 more Quote types
- [ ] 50+ specific index definitions (Euribor tenors, LIBOR variants, SOFR, ESTR, SONIA, USCPI, UKRPI, EUHICP, swap indexes)
- [ ] Port `test-suite/currencies.cpp`, `test-suite/quotes.cpp`

### 🟡 Phase 5 — Term Structures / `ql-termstructures` (~15%)

**Done:** TermStructure trait, YieldTermStructure trait, FlatForward, InterpolatedZeroCurve, InterpolatedDiscountCurve, InterpolatedForwardCurve, VolatilityTermStructure, BlackVolTermStructure, BlackConstantVol, BlackVarianceSurface, LocalVolTermStructure, LocalConstantVol, LocalVolSurface (Dupire), SmileSection (flat, SABR, SVI), smile calibration, DefaultProbabilityTermStructure, FlatHazardRate, InterpolatedHazardRateCurve, inflation term structures.
**Remaining (critical):**
- [ ] **`PiecewiseYieldCurve` bootstrapper** — the most-used term structure
- [ ] `BootstrapTraits` trait (ZeroYield, Discount, ForwardRate)
- [ ] Rate helpers: `DepositRateHelper`, `SwapRateHelper`, `FraRateHelper`, `FuturesRateHelper`
- [ ] `FittedBondDiscountCurve`
- [ ] Swaption vol structures (`SwaptionVolatilityStructure`, `SwaptionVolCube`)
- [ ] Cap/floor vol structures
- [ ] Optionlet vol structures
- [ ] Port 4 C++ test-suite files

### 🟡 Phase 6 — Processes & Models (~15%)

**Done:** StochasticProcess/1D traits, 12 processes (BSM, Heston, HW, G2, Bates, Merton76, OU, SquareRoot, VarianceGamma, GBM, GSR, HullWhiteForward), CalibratedModel, 7 models (Vasicek, HullWhite, CIR, BlackKarasinski, G2++, Heston, Bates).
**Remaining:**
- [ ] Market Model framework (~160 files) — largest single sub-system
- [ ] Additional process/model variants
- [ ] Port 5 C++ test-suite files

### 🔴 Phase 7 — Numerical Methods / `ql-methods` (~5%)

**Done:** Basic BinomialTree (CRR, JR, Tian, LR, Joshi4), TrinomialTree, TimeGrid, backward induction, PathGenerator, AntitheticPathGenerator, MonteCarloModel, TridiagonalOperator, basic 1D FDM solver.
**Remaining:**
- [ ] Multi-dimensional FDM: meshers, operators, schemes (Douglas, Hundsdorfer-Verwer, Craig-Sneyd)
- [ ] FDM solvers: Fdm1DimSolver, FdmNdimSolver
- [ ] Advanced MC: EarlyExercisePathPricer, Longstaff-Schwartz LSM
- [ ] Advanced lattice methods
- [ ] Port 3 C++ test-suite files

### 🟡 Phase 8 — Instruments & Cash Flows (~13%)

**Done:** CashFlow/Coupon traits, SimpleCashFlow, Redemption, FixedRateCoupon/Builder, FloatingRateCoupon/IborCoupon/Builder, CPICoupon, YoYInflationCoupon, CashFlows analytics (NPV, BPS, duration, convexity, z-spread, yield), Instrument trait, Payoff hierarchy, Exercise types, VanillaOption, BarrierOption, Bond (Fixed/Floating/ZeroCoupon), Swap, VanillaSwap, ZeroCouponInflationSwap.
**Remaining:**
- [ ] Cap/Floor, Swaption, CreditDefaultSwap
- [ ] CMS/CMS-spread coupons, range accrual
- [ ] Asian, Lookback, Basket, Cliquet, Quanto options
- [ ] Convertible bonds, amortizing variants
- [ ] FRA, Forward
- [ ] Port 7 C++ test-suite files

### 🔴 Phase 9 — Pricing Engines / `ql-pricingengines` (~5%)

**Done:** AnalyticEuropeanEngine (BSM), AnalyticHestonEngine, BaroneAdesiWhaleyEngine, AnalyticBarrierEngine, DiscountingBondEngine, DiscountingSwapEngine.
**Remaining (164 engines):**
- [ ] MC engines (European, American, Asian, Barrier, Basket)
- [ ] FD engines (Black-Scholes vanilla, Heston barrier, HW swaption)
- [ ] Tree engines (binomial vanilla, callable bond, swaption)
- [ ] Swaption engines (Black, Bachelier, Jamshidian, Tree, G2)
- [ ] Cap/floor engines (Black, Bachelier, Tree, Analytic)
- [ ] Credit engines (MidPointCDS, IntegralCDS, IsdaCDS)
- [ ] Exotic option engines
- [ ] Port 9 C++ test-suite files

### 🔴 Phase 10–11 — Advanced & Experimental

See `plan.md` §17–18 for full scope.

### Phase Completion Checklist

When a phase reaches 100% (all items done, all tests ported and passing):

1. Update `plan.md` §6 — change the phase Status column from 🟡/🔴 to ✅ 100%.
2. Update `plan.md` §6.1 — refresh the overall metrics and per-crate table.
3. Update `INSTRUCTIONS.md` §2 — refresh the high-level metrics table.
4. Update `INSTRUCTIONS.md` §8 — change the phase icon to ✅ and mark all
   remaining items as done.
5. Run `just check` one final time to confirm everything is green.
6. Commit with the message: `phase(N): complete — all tests passing`.

---

## 9. Quality Gates

**Every change must satisfy ALL of the following before being considered complete:**

1. **Compiles cleanly:** `cargo build -p ql-{crate}` — zero warnings.
2. **Tests pass:** `cargo test -p ql-{crate}` — all green.
3. **Clippy clean:** `cargo clippy -p ql-{crate} -- -D warnings` — zero warnings.
4. **Formatted:** `cargo fmt --all -- --check` — passes.
5. **No unsafe:** Unless absolutely necessary with documented rationale.
6. **Numerical equivalence:** Tests compare against known C++ output with documented tolerance.
7. **Doc comments:** All public types, traits, methods, and functions have `///` doc comments.
8. **Module header:** Every file has `//!` module doc referencing C++ origin.

**Batch verification command:**
```bash
just check   # runs fmt-check, clippy, and test
```

**State verification command** (to confirm status tables are accurate):
```bash
# Count source files and lines
find crates/ -name '*.rs' -not -path '*/target/*' | wc -l
find crates/ -name '*.rs' -not -path '*/target/*' -exec cat {} + | wc -l

# Count tests
cargo test --workspace -- --list 2>/dev/null | grep -c '.*: test$'

# Count integration test files
find crates/ -path '*/tests/test_*.rs' | wc -l

# Quick health check
cargo test --workspace 2>&1 | tail -3
```

Run these after updating status tables to verify the numbers match reality.

---

## 10. Mandatory: Keeping `plan.md` and `INSTRUCTIONS.md` in Sync

> **This is not optional.** Both `plan.md` and `INSTRUCTIONS.md` must be
> updated after every meaningful unit of work. They are the project's
> persistent memory — without them, the next session starts blind.

### 10.1 When to Update

| Event | Update `plan.md` | Update `INSTRUCTIONS.md` |
|---|---|---|
| Translated a new C++ module | §6.1 per-crate table, §23 checkboxes | §2.2 per-crate table, §8 phase status |
| Ported a C++ test file | §6.1 metrics, §19 test mapping, §23 | §2.1 metrics, §8 phase status |
| Completed an entire phase | §6 status column → ✅, §6.1 snapshot | §2 metrics, §8 icon → ✅ |
| Added a new crate dependency | §4.3 dependency graph (if changed) | §3.2 dependency graph |
| Discovered a new task/gap | §23 — add new checkbox item | §8 — add to Remaining list |
| Changed a convention or pattern | §3 design mapping (if affected) | §4 conventions section |

### 10.2 What to Update

**In `plan.md`:**
- **§6.1 Current Progress Snapshot** — overall metrics table and per-crate
  breakdown (files, LOC, tests, completeness %).
- **§6 Translation Phases table** — the Status column.
- **§23 Next Steps** — check off completed items, add new ones.
- **§19 Test File Mapping** — mark ported tests.

**In `INSTRUCTIONS.md`:**
- **§2.1 High-Level Metrics** — total files, LOC, tests, integration tests.
- **§2.2 Per-Crate Status** — files, LOC, tests, % done, key gaps.
- **§8 Phase-by-Phase Status** — move items from Remaining → Done.

### 10.3 How to Update (AI workflow)

1. **Before starting work:** Read `INSTRUCTIONS.md` fully. Skim `plan.md` §6.1
   and §23 to understand current state and priorities.
2. **During work:** Track what you translated, what tests you wrote, what new
   gaps you discovered.
3. **After passing quality gates (`just check`):** Update both files in the
   same commit as the implementation. Do not defer this to "later."
4. **Verify accuracy:** The numbers in the tables must match reality. If unsure,
   run `find crates/ -name '*.rs' | wc -l` and `cargo test --workspace 2>&1 | tail -1`
   to get ground-truth counts.

### 10.4 Session Start Protocol

Every new AI session should begin with:

1. **Read `INSTRUCTIONS.md`** — this file, in full. It contains everything
   needed to resume work.
2. **Read `plan.md` §6.1 and §23** — current progress snapshot and prioritized
   next steps.
3. **Run `just check`** — verify the workspace is in a clean state.
4. **Check the Work-in-Progress Log** (§14 at the bottom of this file) — if
   a previous session left unfinished work, continue it before starting
   something new.
5. **Pick the highest-priority unchecked item from `plan.md` §23** — and begin.

This protocol ensures continuity regardless of which AI model or session
continues the work.

> **Tip for the user:** You can start any new AI session with:
>
> *"Read INSTRUCTIONS.md in the workspace root and continue where the last
> session left off."*
>
> The `.github/copilot-instructions.md` file will also auto-prompt the AI
> in VS Code Copilot / Copilot Chat to read these instructions.

---

## 11. Reference: Key File Locations

| What | Path |
|---|---|
| Workspace Cargo.toml | `Cargo.toml` |
| Detailed translation plan | `plan.md` |
| These instructions | `INSTRUCTIONS.md` |
| Core type aliases | `crates/ql-core/src/lib.rs` |
| Error enum + macros | `crates/ql-core/src/errors.rs` |
| Observable/Observer | `crates/ql-core/src/patterns/observable.rs` |
| Handle/RelinkableHandle | `crates/ql-core/src/handle.rs` |
| Date type | `crates/ql-time/src/date.rs` |
| Calendar trait | `crates/ql-time/src/calendar.rs` |
| Interpolation trait | `crates/ql-math/src/interpolations/mod.rs` |
| Array (nalgebra wrapper) | `crates/ql-math/src/array.rs` |
| YieldTermStructure | `crates/ql-termstructures/src/yield_term_structure.rs` |
| Instrument/PricingEngine | `crates/ql-instruments/src/instrument.rs` |
| Example engine (BSM) | `crates/ql-pricingengines/src/analytic_european_engine.rs` |
| Facade re-exports | `quantlib/src/lib.rs` |
| Dev task runner | `justfile` |

---

## 12. Reference: External Crate Usage

| Crate | What It Replaces | Usage Pattern |
|---|---|---|
| `nalgebra` | `Array`, `Matrix`, Cholesky/SVD/QR | Wrap in newtypes; never expose `nalgebra` types in public API |
| `statrs` | Distribution CDF/PDF/InvCDF | Wrap in QuantLib-named types (`CumulativeNormalDistribution`, etc.) |
| `rand` + `rand_distr` | Mersenne Twister, basic RNG | Use via `rand_mt` for MT19937 compatibility |
| `num-traits` | Float, Zero, One bounds | Use for generic numeric code |
| `thiserror` | Error hierarchy | `#[derive(Error)]` on `ql_core::Error` |
| `approx` | Float comparison in tests | `assert_abs_diff_eq!`, `assert_relative_eq!` |
| `chrono` (optional) | Date conversion only | `From<NaiveDate>` / `Into<NaiveDate>` — internal Date stays serial-number |

**What we implement ourselves** (no suitable crate):
- All 24 interpolation schemes (tightly coupled to term-structure API)
- 1D root solvers (Brent, bisection, etc.) — simple, must match C++ behavior exactly
- Optimization framework (CostFunction/Constraint/EndCriteria API)
- Copulas, ODE solvers — niche/tiny
- Sobol direction numbers (if QuantLib's differ from available crates)

---

## 13. Common Pitfalls

### 12.1 Don't Break the Dependency Graph

Before adding a dependency between crates, check the graph in §3.2.
Example: `ql-math` must NOT depend on `ql-time`. If you need `Date` in math
code, that code belongs in `ql-termstructures` or higher.

### 12.2 Interior Mutability — Be Consistent

QuantLib uses `mutable` extensively. In Rust:
- `Cell<f64>` for simple cached scalars.
- `RefCell<Option<T>>` for complex cached values in single-threaded code.
- `Mutex<Option<T>>` for thread-safe variants (behind feature flag).
- Always use `&self` (not `&mut self`) for methods that trigger lazy calculation.

### 12.3 Don't Use `chrono::NaiveDate` Internally

`Date` must be a serial number (`i32`) internally for exact C++ compatibility.
Only provide `From<NaiveDate>` conversion as a convenience.

### 12.4 Match C++ Tolerances Exactly

When porting tests, grep the C++ test for `tolerance`, `accuracy`, `eps`,
`expected` and use the **exact same threshold**. Don't make tests looser or
tighter without documented justification.

### 12.5 Register New Modules in `lib.rs`

Forgetting to add `pub mod foo;` in `lib.rs` will cause the module to be
invisible. Always add both the module declaration and the re-export.

### 12.6 Run the Full Check Before Committing

```bash
just check    # fmt + clippy + test
```

Never leave the workspace in a broken state between steps.

### 12.7 Commit Message Convention

```
phase(N)/crate: short description

- Detailed bullet points
- Maps to C++ files: ql/path/to/file.hpp

Ref: QuantLib commit <hash>
```

Example:
```
phase(2)/ql-time: implement Date and Period types

- Translate ql/time/date.hpp → date.rs
- Translate ql/time/period.hpp → period.rs
- Serial date number arithmetic matches C++ exactly
- 47 tests ported from test-suite/dates.cpp

Ref: QuantLib commit abc1234
```

---

*End of static instructions. The Work-in-Progress Log below is the only
section that changes between sessions.*

---

## 14. Work-in-Progress Log

> **Purpose:** This section is a live handoff area between AI sessions. Before
> ending a session, the AI must record what it was working on, what's done,
> what's not done, and any blockers. The next session reads this first.

### How to Use

**At session end** (or if interrupted), write an entry below with:
- Date
- What was attempted
- What was completed (files created/modified)
- What remains unfinished
- Any blockers or decisions needed from the user

**At session start**, read the latest entry. If work is unfinished, continue
it. If it's done, delete the entry and start fresh from `plan.md` §23.

---

### Current Entry

**Date:** 2025-07-15
**Session:** §23.2E PiecewiseYieldCurve bootstrapper + §23.2F Expand ql-indexes
**Status:** ✅ Complete — 2 new modules, 25 new index definitions, 29 new tests, 904 total passing

**Completed:**
- Created `crates/ql-termstructures/src/rate_helpers.rs` (~632 lines) —
  `RateHelper` trait, `BootstrapCurve` temporary curve view,
  `DepositRateHelper`, `FraRateHelper`, `SwapRateHelper`, `FuturesRateHelper`,
  4 unit tests
- Created `crates/ql-termstructures/src/piecewise_yield_curve.rs` (~666 lines) —
  `PiecewiseYieldCurve` with iterative Brent-based bootstrap, implements
  `TermStructure` + `YieldTermStructure`, custom `brent_bootstrap` solver
  variant that handles FnMut + Option returns, 8 unit tests
  (single deposit, two deposits, deposits+swap, monotone DFs, negative rates,
  empty helpers error, TermStructure trait, YieldTermStructure trait)
- Extended `crates/ql-indexes/src/ibor.rs` — added 10 new IBOR indexes:
  TIBOR, CDOR, BBSW, STIBOR, NIBOR, CIBOR, WIBOR, PRIBOR, BUBOR, JIBAR
  (10 new tests)
- Extended `crates/ql-indexes/src/overnight.rs` — added 4 new overnight indexes:
  TONA, AONIA, CORRA, SARON (4 new tests)
- Extended `crates/ql-indexes/src/swap_index.rs` — added 3 new swap indexes:
  GBP LIBOR Swap, JPY LIBOR Swap, CHF LIBOR Swap (3 new tests)
- Updated lib.rs re-exports for both ql-termstructures and ql-indexes
- Updated plan.md §6.1 snapshot and per-crate metrics
- All 904 workspace tests pass (`just check` clean)

**Files created:**
- `crates/ql-termstructures/src/rate_helpers.rs` (~632 lines)
- `crates/ql-termstructures/src/piecewise_yield_curve.rs` (~666 lines)

**Files modified:**
- `crates/ql-termstructures/src/lib.rs` — added `pub mod rate_helpers;`, `pub mod piecewise_yield_curve;`, re-exports
- `crates/ql-indexes/src/ibor.rs` — added 10 IBOR factory functions + 10 tests
- `crates/ql-indexes/src/overnight.rs` — added 4 overnight factory functions + 4 tests
- `crates/ql-indexes/src/swap_index.rs` — added 3 swap index factory functions + 3 tests
- `crates/ql-indexes/src/lib.rs` — updated re-exports

**Next session should:**
1. Continue §23.2D: add DifferentialEvolution optimizer, SobolBrownianGenerator,
   more interpolation schemes (ConvexMonotone, ABCD, BSpline, Richardson)
2. Or §23.2G: Expand ql-instruments (Cap/Floor, Swaption, FRA, CDS)
3. Or §23.2H: Expand ql-pricingengines (MC engines, binomial, FD Black-Scholes)
