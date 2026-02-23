# QuantLib → Rust: Complete 1:1 Translation Plan

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Source Codebase Inventory](#2-source-codebase-inventory)
3. [C++ → Rust Design Mapping](#3-c--rust-design-mapping)
4. [Crate Architecture](#4-crate-architecture)
5. [Module Mapping (C++ → Rust)](#5-module-mapping-c--rust)
6. [Translation Phases](#6-translation-phases)
7. [Phase 0 — Scaffolding & Infrastructure](#7-phase-0--scaffolding--infrastructure)
8. [Phase 1 — Foundation Layer](#8-phase-1--foundation-layer)
9. [Phase 2 — Time & Calendar System](#9-phase-2--time--calendar-system)
10. [Phase 3 — Math Library](#10-phase-3--math-library)
11. [Phase 4 — Core Financial Primitives](#11-phase-4--core-financial-primitives)
12. [Phase 5 — Term Structures](#12-phase-5--term-structures)
13. [Phase 6 — Processes & Models](#13-phase-6--processes--models)
14. [Phase 7 — Numerical Methods](#14-phase-7--numerical-methods)
15. [Phase 8 — Instruments & Cash Flows](#15-phase-8--instruments--cash-flows)
16. [Phase 9 — Pricing Engines](#16-phase-9--pricing-engines)
17. [Phase 10 — Indexes, Currencies & Quotes](#17-phase-10--indexes-currencies--quotes)
18. [Phase 11 — Experimental Module](#18-phase-11--experimental-module)
19. [Test-First Strategy](#19-test-first-strategy)
20. [C++ Pattern → Rust Idiom Reference](#20-c-pattern--rust-idiom-reference)
21. [Dependency Strategy](#21-dependency-strategy)
22. [Verification & Quality Gates](#22-verification--quality-gates)
23. [Risk Register](#23-risk-register)

---

## 1. Project Overview

**Goal:** Produce a complete, idiomatic, 1:1 translation of the QuantLib C++ library
(`https://github.com/lballabio/QuantLib`) into Rust, preserving every public type,
function, algorithm, and behavioral contract.

**QuantLib in numbers:**

| Metric | Value |
|---|---|
| Core library (ql/) source files | 2,376 (.hpp + .cpp) |
| Core library lines of code | ~310,000 (code only, excl. comments/blanks) |
| Top-level header files (ql/*.hpp) | 45 |
| Top-level modules (ql/*/) | 16 directories |
| Experimental sub-modules | 26 directories (421 files) |
| Test suite files | 182 |
| Test suite lines of code | ~86,600 |
| Calendar implementations | 45 countries |
| Day count conventions | ~12 |
| Pricing engines | ~170 files |
| Instruments | ~86 header files |
| Total (library + tests) | ~400,000 LOC of C++ |

---

## 2. Source Codebase Inventory

### Module size breakdown (recursive file counts)

| C++ Directory | Headers | Sources | Total Files | Translation Priority |
|---|---|---|---|---|
| `ql/*.hpp` (top-level) | 45 | 20 | 65 | **P0** — Core types |
| `ql/patterns/` | 6 | 1 | 7 | **P0** — Observer/Observable, Singleton, LazyObject |
| `ql/utilities/` | 11 | 3 | 14 | **P0** — Null, Clone, DataFormatters |
| `ql/time/` | 73 | 63 | 136 | **P1** — Date, Calendar, DayCounter, Schedule |
| `ql/math/` | 177 | 95 | 272 | **P2** — Distributions, Interpolation, Optimization, RNG, Matrix |
| `ql/currencies/` | 8 | 7 | 15 | **P3** — Currency definitions |
| `ql/quotes/` | 11 | 7 | 18 | **P3** — Market quote wrappers |
| `ql/indexes/` | 67 | 29 | 96 | **P3** — IBOR, Inflation, Swap indexes |
| `ql/termstructures/` | 128 | 73 | 201 | **P4** — Yield, Vol, Credit, Inflation curves |
| `ql/processes/` | 22 | 21 | 43 | **P5** — GBM, Heston, Hull-White processes |
| `ql/models/` | 159 | 121 | 280 | **P5** — Short-rate, Market models, SABR |
| `ql/cashflows/` | 36 | 34 | 70 | **P6** — Coupon, CashFlow, CashFlowVectors |
| `ql/instruments/` | 86 | 81 | 167 | **P6** — Bonds, Swaps, Options, CDS |
| `ql/methods/` | 147 | 90 | 237 | **P7** — FDM, Lattice, Monte Carlo |
| `ql/pricingengines/` | 170 | 134 | 304 | **P8** — All pricing engines |
| `ql/legacy/` | 17 | 13 | 30 | **P9** — LIBOR Market Model legacy |
| `ql/experimental/` | 263 | 158 | 421 | **P10** — Experimental features |

---

## 3. C++ → Rust Design Mapping

This section defines the systematic rules for how every C++ construct maps to Rust.
These rules must be applied consistently across the entire translation.

### 3.1 Fundamental Types

| C++ (QuantLib) | Rust | Notes |
|---|---|---|
| `QL_REAL` / `Real` / `double` | `f64` | Newtype `Real = f64` if we want domain clarity |
| `QL_INTEGER` / `Integer` / `int` | `i32` | |
| `QL_BIG_INTEGER` / `BigInteger` / `long` | `i64` | |
| `Natural` / `unsigned int` | `u32` | |
| `BigNatural` / `unsigned long` | `u64` | |
| `Size` / `std::size_t` | `usize` | |
| `Time` | `f64` (type alias) | Continuous time in year fractions |
| `DiscountFactor` | `f64` (type alias) | |
| `Rate` | `f64` (type alias) | |
| `Spread` | `f64` (type alias) | |
| `Volatility` | `f64` (type alias) | |
| `ext::shared_ptr<T>` | `Arc<T>` or `Rc<T>` | See §3.3 |
| `ext::any` | `Box<dyn Any>` | |
| `ext::optional<T>` | `Option<T>` | Direct mapping |
| `Null<T>()` | `Option<T>` with `None` | Or `f64::NAN` for numerics |
| `std::vector<T>` | `Vec<T>` | |
| `std::map<K,V>` | `BTreeMap<K,V>` or `HashMap<K,V>` | |
| `std::set<T>` | `BTreeSet<T>` or `HashSet<T>` | |
| `std::pair<A,B>` | `(A, B)` | |

### 3.2 Class Hierarchy → Trait Hierarchy

QuantLib uses deep inheritance hierarchies. In Rust these become traits + structs:

```
C++:  class Observable { ... }
      class Observer { ... }
      class LazyObject : public Observable, public Observer { ... }
      class Instrument : public LazyObject { ... }
      class Option : public Instrument { ... }
      class OneAssetOption : public Option { ... }
      class VanillaOption : public OneAssetOption { ... }

Rust: trait Observable { fn register_observer(...); fn notify_observers(&self); }
      trait Observer { fn update(&mut self); }
      trait LazyObject: Observable + Observer { fn calculate(&self); fn perform_calculations(&self); }
      trait Instrument: LazyObject { fn npv(&self) -> f64; fn is_expired(&self) -> bool; }

      struct VanillaOption { ... }
      impl Instrument for VanillaOption { ... }
```

**Key principle:** Leaf classes become structs. Interior classes become traits.
Where C++ uses virtual methods, Rust uses trait methods. Where C++ uses CRTP
(Curiously Recurring Template Pattern), Rust uses generics or associated types.

### 3.3 Smart Pointers & Ownership

| C++ Pattern | Rust Equivalent | When |
|---|---|---|
| `shared_ptr<T>` passed around widely | `Arc<T>` | When T crosses thread boundaries or has multiple owners |
| `shared_ptr<T>` within single-thread context | `Rc<T>` | Default for the Observer graph |
| `shared_ptr<T>` with mutation | `Rc<RefCell<T>>` or `Arc<RwLock<T>>` | LazyObject's mutable cache |
| `Handle<T>` (relinkable) | Custom `Handle<T>` wrapping `Rc<RefCell<Option<Arc<T>>>>` | Preserve semantics exactly |
| Raw `T*` observer back-pointers | `Weak<T>` | Prevent cycles in observer graph |
| `unique_ptr<T>` | `Box<T>` | |
| Passing by `const&` | `&T` | |
| Passing by value | Move (default) or `Clone` | |

### 3.4 Observer/Observable Pattern

This is QuantLib's most pervasive pattern. Every term structure, quote, and instrument
participates. The Rust translation must preserve the same notification semantics:

```rust
// Core traits
pub trait Observable {
    fn register_observer(&self, observer: Weak<dyn Observer>);
    fn unregister_observer(&self, observer: &Weak<dyn Observer>);
    fn notify_observers(&self);
}

pub trait Observer {
    fn update(&self);
}

// Blanket implementation using interior mutability
pub struct ObservableImpl {
    observers: RefCell<Vec<Weak<dyn Observer>>>,
}
```

**Critical considerations:**
- The C++ version has a thread-safe variant (`QL_ENABLE_THREAD_SAFE_OBSERVER_PATTERN`)
  using Boost.Signals2 with mutex. We should provide a feature flag
  `thread-safe-observers` that swaps `Rc`→`Arc`, `RefCell`→`RwLock`.
- Observer graphs can have cycles (instrument ↔ term structure). Use `Weak` references
  for observer back-pointers.

### 3.5 Visitor Pattern

QuantLib uses the Acyclic Visitor pattern extensively (instruments, payoffs, events):

```cpp
// C++
class AcyclicVisitor { virtual ~AcyclicVisitor(); };
template <class T>
class Visitor { virtual void visit(T&) = 0; };
```

Rust translation using trait objects:

```rust
pub trait AcyclicVisitor {}

pub trait Visitor<T> {
    fn visit(&mut self, target: &T);
}

// Or, use Rust enums where the set of visited types is closed:
pub enum Payoff {
    PlainVanilla(PlainVanillaPayoff),
    CashOrNothing(CashOrNothingPayoff),
    // ...
}
```

**Decision rule:** Where QuantLib's visitor visits a **closed set** of types that rarely
changes, prefer a Rust `enum`. Where the set is open-ended (instruments, engines), keep
the trait-based visitor.

### 3.6 Template → Generics

| C++ | Rust |
|---|---|
| `template<class T> class Handle<T>` | `struct Handle<T: Observable>` |
| `template<class ArgumentsType, class ResultsType> class GenericEngine` | `struct GenericEngine<A: Arguments, R: Results>` |
| Template specialization | Trait specialization or separate impl blocks |
| SFINAE / `enable_if` | Trait bounds (`where T: ...`) |
| Template template parameters | Higher-kinded bounds (use associated types) |

### 3.7 Error Handling

| C++ | Rust |
|---|---|
| `QL_REQUIRE(cond, msg)` | `ensure!(cond, msg)` macro → `Result<T, QuantLibError>` |
| `QL_ENSURE(cond, msg)` | Post-condition check → `Result<T, QuantLibError>` |
| `QL_FAIL(msg)` | `return Err(QuantLibError::...)` or `bail!(msg)` |
| Exceptions (`std::runtime_error`) | `thiserror` crate enum |

```rust
#[derive(Debug, thiserror::Error)]
pub enum QuantLibError {
    #[error("precondition failed: {0}")]
    PreconditionFailed(String),
    #[error("postcondition failed: {0}")]
    PostconditionFailed(String),
    #[error("{0}")]
    General(String),
    // domain-specific variants:
    #[error("negative {quantity}: {value}")]
    NegativeValue { quantity: &'static str, value: f64 },
    #[error("null date")]
    NullDate,
    // ...
}
pub type Result<T> = std::result::Result<T, QuantLibError>;
```

### 3.8 Enums & Constants

C++ enums → Rust enums (with `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`).
C++ `static const` → Rust `const` or `lazy_static!`/`std::sync::LazyLock`.

### 3.9 Mutable State & `const` Methods

QuantLib uses `mutable` data members extensively (LazyObject's cached results).
In Rust, this maps to **interior mutability**:

| C++ Pattern | Rust Equivalent |
|---|---|
| `mutable Real NPV_; const calculate()` | `Cell<f64>` or `RefCell<Option<f64>>` inside the struct; `&self` methods |
| `mutable bool calculated_` | `Cell<bool>` |
| `mutable` complex types | `RefCell<T>` |

---

## 4. Crate Architecture

### 4.1 Workspace Layout

```
quantlib-rs/
├── Cargo.toml              # Workspace root
├── flake.nix               # Nix dev environment
├── plan.md                 # This file
├── justfile                # Task runner
│
├── crates/
│   ├── ql-core/            # types, errors, patterns, utilities
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs           ← ql/types.hpp
│   │   │   ├── errors.rs          ← ql/errors.hpp
│   │   │   ├── settings.rs        ← ql/settings.hpp
│   │   │   ├── compounding.rs     ← ql/compounding.hpp
│   │   │   ├── patterns/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── observable.rs  ← ql/patterns/observable.hpp
│   │   │   │   ├── lazy_object.rs ← ql/patterns/lazyobject.hpp
│   │   │   │   ├── singleton.rs   ← ql/patterns/singleton.hpp
│   │   │   │   └── visitor.rs     ← ql/patterns/visitor.hpp
│   │   │   ├── utilities/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── null.rs        ← ql/utilities/null.hpp
│   │   │   │   ├── clone.rs       ← ql/utilities/clone.hpp
│   │   │   │   ├── data_formatters.rs
│   │   │   │   └── data_parsers.rs
│   │   │   └── handle.rs          ← ql/handle.hpp
│   │   └── Cargo.toml
│   │
│   ├── ql-time/            # Date, Calendar, DayCounter, Schedule
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── date.rs            ← ql/time/date.hpp
│   │   │   ├── period.rs          ← ql/time/period.hpp
│   │   │   ├── weekday.rs         ← ql/time/weekday.hpp
│   │   │   ├── frequency.rs       ← ql/time/frequency.hpp
│   │   │   ├── time_unit.rs       ← ql/time/timeunit.hpp
│   │   │   ├── calendar.rs        ← ql/time/calendar.hpp
│   │   │   ├── calendars/         ← ql/time/calendars/  (45 countries)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── united_states.rs
│   │   │   │   ├── united_kingdom.rs
│   │   │   │   ├── target.rs
│   │   │   │   └── ... (one file per country)
│   │   │   ├── day_counter.rs     ← ql/time/daycounter.hpp
│   │   │   ├── day_counters/      ← ql/time/daycounters/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── actual360.rs
│   │   │   │   ├── actual365_fixed.rs
│   │   │   │   ├── actual_actual.rs
│   │   │   │   ├── thirty360.rs
│   │   │   │   └── ...
│   │   │   ├── schedule.rs        ← ql/time/schedule.hpp
│   │   │   ├── date_generation.rs ← ql/time/dategenerationrule.hpp
│   │   │   ├── business_day_convention.rs
│   │   │   ├── imm.rs             ← ql/time/imm.hpp
│   │   │   ├── asx.rs             ← ql/time/asx.hpp
│   │   │   └── ecb.rs             ← ql/time/ecb.hpp
│   │   ├── tests/                  ← ported from test-suite/ (Phase 2)
│   │   │   ├── test_dates.rs       ← test-suite/dates.cpp
│   │   │   ├── test_calendars.rs   ← test-suite/calendars.cpp
│   │   │   ├── test_day_counters.rs← test-suite/daycounters.cpp
│   │   │   └── test_schedule.rs    ← test-suite/schedule.cpp
│   │   └── Cargo.toml
│   │
│   ├── ql-math/            # Math primitives
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── array.rs           ← ql/math/array.hpp (1D vector ops)
│   │   │   ├── matrix.rs          ← ql/math/matrix.hpp
│   │   │   ├── comparison.rs      ← ql/math/comparison.hpp
│   │   │   ├── rounding.rs        ← ql/math/rounding.hpp
│   │   │   ├── functional.rs      ← ql/math/functional.hpp
│   │   │   ├── interpolation.rs   ← ql/math/interpolation.hpp
│   │   │   ├── interpolations/    ← ql/math/interpolations/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── linear.rs
│   │   │   │   ├── cubic.rs
│   │   │   │   ├── log.rs
│   │   │   │   ├── sabr.rs
│   │   │   │   ├── bilinear.rs
│   │   │   │   ├── bicubic_spline.rs
│   │   │   │   └── ... (24 interpolation schemes)
│   │   │   ├── distributions/     ← ql/math/distributions/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── normal.rs
│   │   │   │   ├── bivariate_normal.rs
│   │   │   │   ├── chi_square.rs
│   │   │   │   ├── gamma.rs
│   │   │   │   ├── poisson.rs
│   │   │   │   ├── student_t.rs
│   │   │   │   └── binomial.rs
│   │   │   ├── integrals/         ← ql/math/integrals/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── gauss_kronrod.rs
│   │   │   │   ├── simpson.rs
│   │   │   │   ├── trapezoid.rs
│   │   │   │   └── ...
│   │   │   ├── solvers1d/         ← ql/math/solvers1d/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── brent.rs
│   │   │   │   ├── bisection.rs
│   │   │   │   ├── newton.rs
│   │   │   │   ├── ridder.rs
│   │   │   │   └── ...
│   │   │   ├── optimization/      ← ql/math/optimization/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── cost_function.rs
│   │   │   │   ├── constraint.rs
│   │   │   │   ├── end_criteria.rs
│   │   │   │   ├── levenberg_marquardt.rs
│   │   │   │   ├── bfgs.rs
│   │   │   │   ├── conjugate_gradient.rs
│   │   │   │   ├── differential_evolution.rs
│   │   │   │   ├── simplex.rs
│   │   │   │   └── ...
│   │   │   ├── random_numbers/    ← ql/math/randomnumbers/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── mersenne_twister.rs
│   │   │   │   ├── sobol.rs
│   │   │   │   ├── halton.rs
│   │   │   │   ├── inverse_cumulative.rs
│   │   │   │   └── ...
│   │   │   ├── statistics/        ← ql/math/statistics/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── general_statistics.rs
│   │   │   │   ├── incremental_statistics.rs
│   │   │   │   ├── convergence_statistics.rs
│   │   │   │   └── ...
│   │   │   ├── matrix_utilities/  ← ql/math/matrixutilities/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── cholesky.rs
│   │   │   │   ├── svd.rs
│   │   │   │   ├── qr.rs
│   │   │   │   ├── eigenvalues.rs
│   │   │   │   └── ...
│   │   │   ├── copulas/           ← ql/math/copulas/
│   │   │   │   ├── mod.rs
│   │   │   │   └── ...
│   │   │   └── ode/               ← ql/math/ode/
│   │   │       └── ...
│   │   └── Cargo.toml
│   │
│   ├── ql-termstructures/  # Yield, Vol, Credit, Inflation curves
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── term_structure.rs  ← ql/termstructure.hpp (base)
│   │   │   ├── yield_/           ← ql/termstructures/yield/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── flat_forward.rs
│   │   │   │   ├── zero_curve.rs
│   │   │   │   ├── discount_curve.rs
│   │   │   │   ├── piecewise_yield_curve.rs
│   │   │   │   └── ...
│   │   │   ├── volatility/       ← ql/termstructures/volatility/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── equityfx/
│   │   │   │   ├── swaption/
│   │   │   │   ├── capfloor/
│   │   │   │   ├── optionlet/
│   │   │   │   └── inflation/
│   │   │   ├── credit/           ← ql/termstructures/credit/
│   │   │   │   └── ...
│   │   │   └── inflation/        ← ql/termstructures/inflation/
│   │   │       └── ...
│   │   └── Cargo.toml
│   │
│   ├── ql-currencies/      # Currency + ExchangeRate + Money
│   │   └── ...
│   │
│   ├── ql-indexes/         # Index, IBOR, Inflation, Swap
│   │   └── ...
│   │
│   ├── ql-quotes/          # Quote, SimpleQuote, etc.
│   │   └── ...
│   │
│   ├── ql-cashflows/       # CashFlow, Coupon, Legs
│   │   └── ...
│   │
│   ├── ql-processes/       # StochasticProcess, GBM, Heston, etc.
│   │   └── ...
│   │
│   ├── ql-models/          # Short-rate, Market Model, Equity models
│   │   └── ...
│   │
│   ├── ql-instruments/     # Instrument, Bond, Swap, Option, CDS, etc.
│   │   └── ...
│   │
│   ├── ql-methods/         # FDM, Lattice, Monte Carlo
│   │   ├── src/
│   │   │   ├── finite_differences/
│   │   │   ├── lattices/
│   │   │   └── monte_carlo/
│   │   └── Cargo.toml
│   │
│   ├── ql-pricingengines/  # All pricing engines
│   │   ├── src/
│   │   │   ├── vanilla/
│   │   │   ├── barrier/
│   │   │   ├── asian/
│   │   │   ├── bond/
│   │   │   ├── swap/
│   │   │   ├── swaption/
│   │   │   ├── capfloor/
│   │   │   ├── credit/
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   ├── ql-experimental/    # Experimental features (feature-gated)
│   │   └── ...
│   │
│   └── ql-legacy/          # Legacy LIBOR Market Model
│       └── ...
│
├── quantlib/               # Facade crate: re-exports everything
│   ├── src/lib.rs          # `pub use ql_core::*; pub use ql_time::*; ...`
│   └── Cargo.toml
│
└── benches/                # Criterion benchmarks (cross-crate)
    ├── bench_black_scholes.rs
    ├── bench_matrix.rs
    └── ...
```

> **Note:** Tests live inside each crate (`crates/ql-*/tests/`) rather than a
> top-level `tests/` directory. This ensures each crate can be tested independently
> and tests are ported alongside the implementation (see §19).

### 4.2 Why a Workspace with Multiple Crates?

1. **Parallel compilation** — Cargo compiles independent crates in parallel.
   `ql-math` and `ql-time` don't depend on each other and compile simultaneously.
2. **Incremental development** — Each phase adds one or two crates. The project
   compiles and passes tests at every phase boundary.
3. **Clean dependency graph** — Forces us to avoid circular dependencies, which is
   exactly the layering QuantLib already has.
4. **Optional features** — `ql-experimental` is behind a feature flag. Users who
   don't need it save compile time.

### 4.3 Dependency Graph Between Crates

```
ql-core  (zero deps, the foundation)
  ↑
  ├── ql-time  (depends on ql-core)
  ├── ql-math  (depends on ql-core)
  │     ↑
  │     ├── ql-currencies  (ql-core, ql-time)
  │     ├── ql-quotes      (ql-core)
  │     ├── ql-indexes     (ql-core, ql-time, ql-currencies)
  │     │
  │     ├── ql-termstructures  (ql-core, ql-time, ql-math, ql-quotes)
  │     │     ↑
  │     │     ├── ql-processes  (ql-core, ql-math, ql-termstructures)
  │     │     │     ↑
  │     │     │     └── ql-models  (ql-core, ql-math, ql-processes, ql-termstructures)
  │     │     │
  │     │     ├── ql-cashflows  (ql-core, ql-time, ql-math, ql-indexes, ql-termstructures)
  │     │     │     ↑
  │     │     │     └── ql-instruments  (ql-core, ql-time, ql-cashflows, ql-termstructures)
  │     │     │           ↑
  │     │     │           └── ql-pricingengines  (everything above)
  │     │     │
  │     │     └── ql-methods  (ql-core, ql-math, ql-processes, ql-termstructures)
  │     │
  │     └── ql-experimental  (depends on everything)
  │
  └── ql-legacy  (ql-core, ql-math, ql-models)

quantlib (facade) → re-exports all of the above
```

---

## 5. Module Mapping (C++ → Rust)

### 5.1 Complete File Mapping Reference

Every C++ header/source pair maps to exactly one Rust module file. The mapping
convention is:

| C++ File | Rust Module |
|---|---|
| `ql/foo_bar.hpp` + `ql/foo_bar.cpp` | `crates/ql-core/src/foo_bar.rs` |
| `ql/time/calendars/united_states.hpp/.cpp` | `crates/ql-time/src/calendars/united_states.rs` |
| `ql/math/interpolations/linear_interpolation.hpp` | `crates/ql-math/src/interpolations/linear.rs` |

**Naming convention:** C++ `camelCase` file names → Rust `snake_case` file names.
C++ class names stay `PascalCase` in Rust (they become struct/trait names).

---

## 6. Translation Phases

The translation is ordered by dependency depth — we build from the leaves inward.
Each phase is self-contained: it compiles, all tests pass, and it can be reviewed
independently.

**Test-first rule:** Every phase ports the corresponding C++ test-suite files
*before or alongside* the implementation. Tests are the specification — they tell
us when the translation is correct. No phase is complete until its tests pass.

| Phase | Name | Crates | Est. Impl Files | Test Files Ported | Depends On |
|---|---|---|---|---|---|
| **0** | Scaffolding | workspace, CI | — | — | — |
| **1** | Foundation | `ql-core` | ~80 | `errors.cpp`, `observable.cpp` | — |
| **2** | Time & Calendar | `ql-time` | ~136 | `dates.cpp`, `calendars.cpp`, `daycounters.cpp`, `schedule.cpp` | Phase 1 |
| **3** | Math Library | `ql-math` | ~272 | `matrices.cpp`, `array.cpp`, `interpolations.cpp`, `distributions.cpp`, `solvers1d.cpp`, `optimizers.cpp`, `rngtraits.cpp`, `statistics.cpp`, `lowdiscrepancysequences.cpp` | Phase 1 |
| **4** | Core Financial Primitives | `ql-currencies`, `ql-quotes`, `ql-indexes` | ~129 | `currencies.cpp`, `quotes.cpp` | Phases 1–3 |
| **5** | Term Structures | `ql-termstructures` | ~201 | `termstructures.cpp`, `interestrateindex.cpp`, `piecewiseyieldcurve.cpp`, `fittedbonddiscountcurve.cpp`, `swaptionvolatilitymatrix.cpp` | Phases 1–4 |
| **6** | Processes & Models | `ql-processes`, `ql-models` | ~323 | `hestonmodel.cpp`, `shortratemodels.cpp`, `marketmodel.cpp`, `marketmodel_smm.cpp`, `marketmodel_cms.cpp` | Phases 1–5 |
| **7** | Numerical Methods | `ql-methods` | ~237 | `fdm.cpp`, `batesmodel.cpp` (uses FDM), `latticemethods.cpp` | Phases 1–6 |
| **8** | Instruments & Cash Flows | `ql-cashflows`, `ql-instruments` | ~237 | `bonds.cpp`, `swaps.cpp`, `overnightindexedswap.cpp`, `capfloor.cpp`, `swaptions.cpp`, `creditdefaultswap.cpp`, `cashflows.cpp` | Phases 1–7 |
| **9** | Pricing Engines | `ql-pricingengines` | ~304 | `europeanoption.cpp`, `americanoption.cpp`, `asianoptions.cpp`, `barrieroption.cpp`, `lookbackoptions.cpp`, `basketoption.cpp`, `cliquetoption.cpp`, `quantooption.cpp`, `forwardoption.cpp` | Phases 1–8 |
| **10** | Indexes, Currencies & Quotes (advanced) | enrich earlier crates | ~50 | `inflation.cpp`, `inflationcpibond.cpp`, `inflationcpiswap.cpp` | Phases 1–9 |
| **11** | Experimental | `ql-experimental` | ~421 | `variancegamma.cpp`, `varianceoption.cpp`, `catbonds.cpp`, remaining experimental tests | Phases 1–9 |

---

## 7. Phase 0 — Scaffolding & Infrastructure

### Tasks

- [x] `flake.nix` — Nix development environment (done)
- [ ] Initialize Cargo workspace with all crate stubs
- [ ] Set up `justfile` with common commands
- [ ] Configure CI (GitHub Actions): `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`
- [ ] Set up `cargo-llvm-cov` for coverage reporting
- [ ] Clone QuantLib reference at pinned commit as git submodule under `reference/`
- [ ] Add `.gitignore`, `LICENSE` (BSD 3-Clause, matching QuantLib), `README.md`
- [ ] Configure `rustfmt.toml` and `clippy.toml`

### Deliverables

```
quantlib-rs/
├── Cargo.toml          # [workspace] members = ["crates/*", "quantlib"]
├── flake.nix
├── justfile
├── .github/workflows/ci.yml
├── reference/          # git submodule → QuantLib at pinned commit
├── crates/
│   ├── ql-core/Cargo.toml
│   ├── ql-time/Cargo.toml
│   ├── ql-math/Cargo.toml
│   └── ... (all stubs)
└── quantlib/Cargo.toml
```

---

## 8. Phase 1 — Foundation Layer

**C++ sources:** `ql/*.hpp` (top-level), `ql/patterns/`, `ql/utilities/`
**Rust crate:** `ql-core`
**~80 files → ~40 Rust modules**

### Test files to port first

| C++ Test File | Tests | What It Validates |
|---|---|---|
| `test-suite/observable.cpp` | Observer registration, notification, deregistration, circular refs | Observer/Observable/Handle core |
| `test-suite/errors.cpp` | `QL_REQUIRE`, `QL_ENSURE`, `QL_FAIL` behavior, error messages | Error handling macros |

**Approach:** Write the Rust `#[test]` functions first (they will fail to compile).
Then implement types until the tests compile and pass. This is pure TDD.

### 8.1 Core Types (`types.rs`)

```rust
// crates/ql-core/src/types.rs

/// Integer number (maps to C++ QL_INTEGER = int)
pub type Integer = i32;

/// Large integer number (maps to C++ QL_BIG_INTEGER = long)
pub type BigInteger = i64;

/// Positive integer
pub type Natural = u32;

/// Large positive integer
pub type BigNatural = u64;

/// Real number (maps to C++ QL_REAL = double)
pub type Real = f64;

/// Decimal number (alias for Real)
pub type Decimal = f64;

/// Size of a container
pub type Size = usize;

/// Continuous quantity with 1-year units
pub type Time = f64;

/// Discount factor between dates
pub type DiscountFactor = f64;

/// Interest rate
pub type Rate = f64;

/// Spread over a reference rate
pub type Spread = f64;

/// Volatility
pub type Volatility = f64;
```

### 8.2 Modules in Phase 1

| Rust Module | C++ Origin | Description |
|---|---|---|
| `types.rs` | `ql/types.hpp`, `ql/qldefines.hpp` | Fundamental type aliases |
| `errors.rs` | `ql/errors.hpp` | `QuantLibError` enum, `ql_require!`, `ql_ensure!`, `ql_fail!` macros |
| `compounding.rs` | `ql/compounding.hpp` | `Compounding` enum |
| `position.rs` | `ql/position.hpp` | `Position::Type` enum |
| `settings.rs` | `ql/settings.hpp` | Global evaluation date (thread-local) |
| `handle.rs` | `ql/handle.hpp` | `Handle<T>`, `RelinkableHandle<T>` |
| `patterns/observable.rs` | `ql/patterns/observable.hpp` | `Observable`, `Observer` traits + impl |
| `patterns/lazy_object.rs` | `ql/patterns/lazyobject.hpp` | `LazyObject` trait |
| `patterns/singleton.rs` | `ql/patterns/singleton.hpp` | `Singleton<T>` (→ `LazyLock<T>`) |
| `patterns/visitor.rs` | `ql/patterns/visitor.hpp` | `AcyclicVisitor`, `Visitor<T>` |
| `utilities/null.rs` | `ql/utilities/null.hpp` | `Null` trait (→ `Default` or `Option`) |
| `utilities/clone.rs` | `ql/utilities/clone.hpp` | `CloneableIntoExt` (→ Rust `Clone`) |
| `utilities/data_formatters.rs` | `ql/utilities/dataformatters.hpp` | Formatting helpers |
| `utilities/data_parsers.rs` | `ql/utilities/dataparsers.hpp` | Parsing helpers |

### 8.3 Key Decisions

1. **`Settings` singleton:** Use `thread_local!` with `RefCell<Settings>` — matches
   QuantLib's pre-thread-safe behavior. Provide a `scoped_evaluation_date()` RAII guard.

2. **`Handle<T>`:** Implement as:
   ```rust
   pub struct Handle<T: ?Sized> {
       link: Rc<RefCell<Link<T>>>,
   }
   pub struct RelinkableHandle<T: ?Sized> {
       link: Rc<RefCell<Link<T>>>,
   }
   ```
   Where `Link<T>` holds `Option<Arc<T>>` and an observer set.

---

## 9. Phase 2 — Time & Calendar System

**C++ sources:** `ql/time/` (73 headers, 63 sources)
**Rust crate:** `ql-time`
**136 files → ~75 Rust modules**

### Test files to port first

| C++ Test File | Tests | What It Validates |
|---|---|---|
| `test-suite/dates.cpp` | Serial number round-trips, arithmetic, IMM/ASX/ECB dates, leap years | `Date`, `Period`, `IMM`, `ASX`, `ECB` |
| `test-suite/calendars.cpp` | Holiday lists for every country, advance/adjust, joint calendars | All 45 `Calendar` implementations |
| `test-suite/daycounters.cpp` | Year fraction for every convention, edge cases | All `DayCounter` implementations |
| `test-suite/schedule.cpp` | Forward/backward generation, stubs, end-of-month, CDS schedules | `Schedule` builder |

**Approach:** Port all four test files first. This gives ~200+ test cases that act as
the acceptance gate for the entire time module.

### 9.1 Key Types

| Rust Type | C++ Type | Notes |
|---|---|---|
| `Date` | `Date` | Serial date number (i32 days since epoch). NOT chrono—match QuantLib exactly. |
| `Period` | `Period` | Length + TimeUnit |
| `TimeUnit` | `TimeUnit` enum | Days, Weeks, Months, Years |
| `Weekday` | `Weekday` enum | Sunday=1 … Saturday=7 |
| `Month` | `Month` enum | January=1 … December=12 |
| `Frequency` | `Frequency` enum | NoFrequency, Once, Annual, … Daily |
| `DateGeneration::Rule` | `DateGeneration::Rule` | Forward, Backward, Zero, etc. |
| `BusinessDayConvention` | `BusinessDayConvention` | Following, ModifiedFollowing, etc. |
| `Calendar` (trait) | `Calendar` (virtual class) | `is_business_day()`, `advance()`, `adjust()` |
| `DayCounter` (trait) | `DayCounter` (virtual class) | `day_count()`, `year_fraction()` |
| `Schedule` | `Schedule` | Date schedule generator |
| `IMM` | `IMM` | IMM date logic |
| `ASX` | `ASX` | ASX date logic |
| `ECB` | `ECB` | ECB date logic |

### 9.2 Calendar Implementations (one per file)

All 45 country calendars from `ql/time/calendars/`:

Argentina, Australia, Austria, Botswana, Brazil, Canada, Chile, China,
Czech Republic, Denmark, Finland, France, Germany, Hong Kong, Hungary,
Iceland, India, Indonesia, Israel, Italy, Japan, Mexico, New Zealand,
Norway, Poland, Romania, Russia, Saudi Arabia, Singapore, Slovakia,
South Africa, South Korea, Sweden, Switzerland, Taiwan, TARGET, Thailand,
Turkey, Ukraine, United Kingdom, United States, Weekends-Only,
Bespoke Calendar, Joint Calendar, Null Calendar.

### 9.3 Day Counter Implementations

Actual/360, Actual/364, Actual/365.25, Actual/365 Fixed, Actual/366,
Actual/Actual (ISDA, ISMA, AFB, Bond), Business/252, 1/1,
Simple, Thirty/360 (variants), Thirty/365.

### 9.4 Translation Notes

- `Date` in C++ uses a serial number (days from a fixed epoch). Reproduce this
  exactly — do NOT use `chrono::NaiveDate` as the internal representation. Provide
  conversion `From<chrono::NaiveDate>` as a convenience.
- `Calendar` in C++ uses pimpl (Bridge pattern): `Calendar` holds a `shared_ptr<Impl>`.
  In Rust: `Calendar` is an enum wrapping concrete calendar types, or a trait object
  `Box<dyn CalendarImpl>`.
- `DayCounter` follows the same Bridge/pimpl pattern → same treatment.

---

## 10. Phase 3 — Math Library

**C++ sources:** `ql/math/` (177 headers, 95 sources)
**Rust crate:** `ql-math`
**272 files → ~130 Rust modules**

### Test files to port first

| C++ Test File | Tests | What It Validates |
|---|---|---|
| `test-suite/matrices.cpp` | Multiply, transpose, determinant, inverse, Cholesky, SVD, QR, pseudo-sqrt | `Matrix`, `Array`, matrix utilities |
| `test-suite/array.cpp` | Element-wise ops, dot product, norms, sorting | `Array` |
| `test-suite/interpolations.cpp` | All interpolation schemes, derivatives, boundary conditions | All `Interpolation` implementations |
| `test-suite/distributions.cpp` | Normal, bivariate normal, chi-square, gamma, Student-t, Poisson, binomial | All `Distribution` implementations |
| `test-suite/solvers1d.cpp` | Brent, bisection, Newton, Ridder, secant, false-position, Halley | All 1D root-finders |
| `test-suite/optimizers.cpp` | Levenberg-Marquardt, BFGS, conjugate gradient, simplex, diff. evolution | All optimizers |
| `test-suite/rngtraits.cpp` | MT19937 sequences, Sobol dimensions, Halton | All RNG implementations |
| `test-suite/lowdiscrepancysequences.cpp` | Sobol convergence, dimension bounds | Quasi-random sequences |
| `test-suite/statistics.cpp` | Mean, variance, skewness, kurtosis, percentiles | Statistics accumulators |
| `test-suite/integrals.cpp` | Gauss-Kronrod, Simpson, trapezoid, Gauss-Lobatto | Numerical integration |

**Approach:** This is the largest test surface (~400+ individual tests). Port them in
sub-batches: matrices/arrays first, then distributions, then interpolation, then
solvers/optimization, then RNG/statistics.

### 10.1 Sub-modules

| Sub-module | Files | Key Types |
|---|---|---|
| Root (`math/`) | ~30 | `Array`, `Matrix`, `Rounding`, `Comparison` |
| `distributions/` | 15 | `NormalDistribution`, `CumulativeNormalDistribution`, `InverseCumulativeNormal`, `BivariateCumulativeNormalDistribution`, `ChiSquareDistribution`, `GammaDistribution`, `StudentTDistribution`, `PoissonDistribution`, `BinomialDistribution` |
| `interpolations/` | 24 | `LinearInterpolation`, `CubicInterpolation`, `LogLinearInterpolation`, `SABRInterpolation`, `BilinearInterpolation`, `BicubicSplineInterpolation`, `BackwardFlatInterpolation`, `ForwardFlatInterpolation`, `LagrangeInterpolation`, `ChebyshevInterpolation`, `ConvexMonotoneInterpolation`, etc. |
| `integrals/` | ~15 | `GaussKronrodAdaptive`, `SimpsonIntegral`, `TrapezoidIntegral`, `GaussLobattoIntegral`, `GaussianQuadratures`, etc. |
| `solvers1d/` | 10 | `Brent`, `Bisection`, `Newton`, `NewtonSafe`, `Ridder`, `Secant`, `FalsePosition`, `Halley`, `FiniteDifferenceNewtonSafe` |
| `optimization/` | ~35 | `CostFunction`, `Constraint`, `EndCriteria`, `LevenbergMarquardt`, `BFGS`, `ConjugateGradient`, `Simplex`, `DifferentialEvolution`, `SimulatedAnnealing`, `ParticleSwarmOptimization`, `Problem`, `Armijo`, `Goldstein` |
| `randomnumbers/` | ~25 | `MersenneTwister`, `SobolRsg`, `HaltonRsg`, `InverseCumulativeRng`, `RandomSequenceGenerator`, `KnuthUniformRng`, etc. |
| `statistics/` | ~10 | `GeneralStatistics`, `IncrementalStatistics`, `ConvergenceStatistics`, `RiskStatistics` |
| `matrixutilities/` | ~15 | `CholeskyDecomposition`, `SVD`, `QRDecomposition`, `SymmetricSchurDecomposition`, `PseudoSqrt`, `BiCGstab` |
| `copulas/` | ~8 | `GaussianCopula`, `ClaytonCopula`, `MinCopula`, `MaxCopula`, etc. |
| `ode/` | ~3 | `AdaptiveRungeKutta` |

### 10.2 Translation Notes

- **`Array`**: Newtype around `nalgebra::DVector<f64>`. Provides the same API
  surface as QuantLib's `Array` (`Index`, `Add`, `Sub`, `Mul`, `Div`, `Neg`,
  `dot_product`). All heavy lifting (SIMD, cache-friendly iteration) comes from
  nalgebra for free. See §21.2 for the wrapper pattern.

- **`Matrix`**: Newtype around `nalgebra::DMatrix<f64>`. Delegates to nalgebra for
  transpose, multiply, determinant, inverse. Matrix utilities (Cholesky, SVD, QR,
  eigenvalues, pseudo-sqrt) map directly to `nalgebra::linalg::*`.

- **Distributions**: Delegate to `statrs` for PDF, CDF, and inverse CDF of
  Normal, Chi-Square, Gamma, Student-t, Poisson, Binomial. Wrap in our own types
  to match QuantLib's API names (`CumulativeNormalDistribution`, etc.). The
  `BivariateCumulativeNormalDistribution` is specialized enough that we translate
  it directly from C++.

- **Random numbers**: Use `rand::rngs::StdRng` (ChaCha) as the default and
  `rand::rngs::SmallRng` for speed-critical Monte Carlo. For reproducibility with
  C++ QuantLib test output, provide `Mt19937` via the `rand_mt` crate (same
  algorithm, same state). Sobol sequences: use `sobol_burley` or ship QuantLib's
  Joe-Kuo direction number table with a thin generator.

- **Interpolation**: No existing Rust crate matches QuantLib's interpolation API
  (evaluate + primitive + derivative + second_derivative + update). Translate these
  from C++ directly:
  ```rust
  pub trait Interpolation {
      fn value(&self, x: f64) -> Result<f64>;
      fn primitive(&self, x: f64) -> Result<f64>;
      fn derivative(&self, x: f64) -> Result<f64>;
      fn second_derivative(&self, x: f64) -> Result<f64>;
  }
  ```

- **1D Solvers & Optimization**: Translate from C++ directly. These are tightly
  coupled to QuantLib's `CostFunction`/`Constraint`/`EndCriteria` API and are
  simple enough (~50-200 LOC each) that wrapping an external crate would be
  more complex than a direct translation.

---

## 11. Phase 4 — Core Financial Primitives

**Rust crates:** `ql-currencies`, `ql-quotes`, `ql-indexes`

### 11.1 Currencies (`ql-currencies`, 15 files)

| C++ | Rust |
|---|---|
| `Currency` class | `Currency` struct |
| `Money` class | `Money` struct |
| `ExchangeRate` class | `ExchangeRate` struct |
| `currencies/africa.hpp` | `currencies/africa.rs` |
| `currencies/america.hpp` | `currencies/america.rs` |
| `currencies/asia.hpp` | `currencies/asia.rs` |
| `currencies/europe.hpp` | `currencies/europe.rs` |
| `currencies/oceania.hpp` | `currencies/oceania.rs` |
| `currencies/crypto.hpp` | `currencies/crypto.rs` |

### 11.2 Quotes (`ql-quotes`, 18 files)

- `Quote` trait (← `Quote` abstract class)
- `SimpleQuote` struct
- `DerivedQuote`, `CompositeQuote`, `ForwardValueQuote`, `ImpliedStdDevQuote`, etc.

### 11.3 Indexes (`ql-indexes`, 96 files)

- `Index` trait (← `Index` abstract class)
- `InterestRateIndex` trait
- IBOR: `Euribor`, `USDLibor`, `GBPLibor`, `JPYLibor`, `Sofr`, `Estr`, etc.
- Inflation indexes: `USCPI`, `UKRPI`, `EUHICP`, etc.
- Swap indexes: `EuriborSwapIsdaFixA`, `UsdLiborSwapIsdaFixAm`, etc.

---

## 12. Phase 5 — Term Structures

**C++ sources:** `ql/termstructure.hpp` + `ql/termstructures/` (128 headers, 73 sources)
**Rust crate:** `ql-termstructures`
**~201 files → ~100 Rust modules**

### 12.1 Hierarchy

```
TermStructure (trait)
├── YieldTermStructure (trait)
│   ├── FlatForward
│   ├── ZeroCurve
│   ├── DiscountCurve
│   ├── ForwardCurve
│   ├── InterpolatedZeroCurve<I>
│   ├── InterpolatedDiscountCurve<I>
│   ├── InterpolatedForwardCurve<I>
│   ├── PiecewiseYieldCurve<Traits, I>   ← bootstrapper
│   ├── FittedBondDiscountCurve
│   └── ...
├── VolatilityTermStructure (trait)
│   ├── BlackVolTermStructure (trait)
│   │   ├── BlackConstantVol
│   │   ├── BlackVarianceSurface
│   │   └── ...
│   ├── LocalVolTermStructure (trait)
│   │   ├── LocalConstantVol
│   │   ├── LocalVolSurface
│   │   └── ...
│   ├── SwaptionVolatilityStructure (trait)
│   │   ├── ConstantSwaptionVolatility
│   │   ├── SwaptionVolCube1
│   │   ├── SwaptionVolCube2
│   │   └── ...
│   ├── CapFloorTermVolatilityStructure (trait)
│   └── OptionletVolatilityStructure (trait)
├── DefaultProbabilityTermStructure (trait)
│   ├── FlatHazardRate
│   ├── InterpolatedHazardRateCurve
│   ├── InterpolatedDefaultDensityCurve
│   ├── PiecewiseDefaultCurve
│   └── ...
└── InflationTermStructure (trait)
    ├── ZeroInflationTermStructure
    ├── YoYInflationTermStructure
    └── ...
```

### 12.2 Bootstrap Engine

The `PiecewiseYieldCurve<Traits, Interpolator>` and `PiecewiseDefaultCurve` use an
iterative bootstrap algorithm. This is one of the most template-heavy parts of
QuantLib. In Rust:

```rust
pub struct PiecewiseYieldCurve<T: BootstrapTraits, I: Interpolator> {
    // ...
}
```

Where `BootstrapTraits` is a trait defining what the curve bootstraps (zero rates,
discount factors, forward rates) and `Interpolator` is a trait for the interpolation
scheme.

---

## 13. Phase 6 — Processes & Models

### 13.1 Stochastic Processes (`ql-processes`, 43 files)

```
StochasticProcess (trait)
├── StochasticProcess1D (trait)
│   ├── GeneralizedBlackScholesProcess
│   ├── BlackScholesProcess
│   ├── BlackScholesMertonProcess
│   ├── GeometricBrownianMotionProcess
│   ├── OrnsteinUhlenbeckProcess
│   ├── SquareRootProcess
│   ├── Merton76Process
│   ├── VarianceGammaProcess
│   └── ...
├── HestonProcess
├── BatesProcess
├── HullWhiteProcess
├── G2Process
├── HullWhiteForwardProcess
├── GsrProcess
└── ...
```

### 13.2 Models (`ql-models`, 280 files)

```
CalibratedModel (trait)
├── ShortRateModel (trait)
│   ├── OneFactorModel (trait)
│   │   ├── HullWhite
│   │   ├── BlackKarasinski
│   │   ├── Vasicek
│   │   ├── CoxIngersollRoss
│   │   └── ...
│   └── TwoFactorModel (trait)
│       ├── G2
│       └── ...
├── AffineModel (trait)
│   └── ...
└── EquityModel
    ├── HestonModel
    ├── BatesModel
    └── ...

MarketModel (trait)            ← LIBOR Market Model framework
├── AbcdVol
├── FlatVol
├── PseudoRootFacade
└── ...
```

The Market Model framework is the most complex sub-module (~160 files including
brownian generators, evolvers, products, path-wise greeks). It requires careful
attention to Array/Matrix semantics.

---

## 14. Phase 7 — Numerical Methods

**C++ sources:** `ql/methods/` (147 headers, 90 sources)
**Rust crate:** `ql-methods`

### 14.1 Finite Difference Methods (the largest sub-module)

```
ql/methods/finitedifferences/
├── meshers/        — Fdm1dMesher, FdmMesherComposite, etc.
├── operators/      — FdmLinearOp, TripleBandLinearOp, etc.
├── schemes/        — Douglas, CraigSneyd, Hundsdorfer-Verwer, etc.
├── solvers/        — FdmBackwardSolver, Fdm1DimSolver, FdmNdimSolver
├── stepconditions/ — FdmStepConditionComposite, etc.
└── utilities/      — FdmIndicesOnBoundary, FdmQuantoHelper, etc.
```

### 14.2 Lattice Methods

```
Lattice (trait)
├── TreeLattice (trait)
│   ├── BinomialTree (CRR, JR, Trigeorgis, Tian, LeisenReimer, Joshi4)
│   ├── TrinomialTree
│   └── ...
└── TreeLattice1D, TreeLattice2D
```

### 14.3 Monte Carlo

```
MonteCarloModel<MC, RNG, S>
├── PathPricer (trait)
├── PathGenerator
├── EarlyExercisePathPricer
└── ...
```

---

## 15. Phase 8 — Instruments & Cash Flows

### 15.1 Cash Flows (`ql-cashflows`, 70 files)

```
CashFlow (trait)
├── SimpleCashFlow
├── Coupon (trait)
│   ├── FixedRateCoupon
│   ├── FloatingRateCoupon
│   │   ├── IborCoupon
│   │   ├── CmsCoupon
│   │   ├── CmsSpreadCoupon
│   │   └── ...
│   ├── InflationCoupon
│   │   ├── CPICoupon
│   │   ├── YoYInflationCoupon
│   │   └── ...
│   └── ...
├── CashFlows (utility: static methods for NPV, BPS, yield, duration, convexity)
└── Leg (= Vec<Box<dyn CashFlow>>)
```

### 15.2 Instruments (`ql-instruments`, 167 files)

```
Instrument (trait, extends LazyObject)
├── Bond
│   ├── ZeroCouponBond
│   ├── FixedRateBond
│   ├── FloatingRateBond
│   ├── AmortizingBond variants
│   ├── ConvertibleBond variants
│   └── ...
├── Swap
│   ├── VanillaSwap
│   ├── NonstandardSwap
│   ├── OvernightIndexedSwap
│   ├── ArithmeticAverageOIS
│   └── ...
├── Option (trait)
│   ├── OneAssetOption (trait)
│   │   ├── VanillaOption
│   │   ├── EuropeanOption
│   │   └── ...
│   ├── BarrierOption, DoubleBarrierOption
│   ├── AsianOption
│   ├── BasketOption
│   ├── CliquetOption
│   ├── LookbackOption
│   ├── ForwardVanillaOption
│   └── ...
├── CapFloor (Cap, Floor, Collar)
├── Swaption
├── CreditDefaultSwap
├── Forward, FRA
└── ...
```

Also includes:
- `Payoff` hierarchy (PlainVanilla, CashOrNothing, AssetOrNothing, Gap, SuperShare, etc.)
- `Exercise` types (European, American, Bermudan)

---

## 16. Phase 9 — Pricing Engines

**C++ sources:** `ql/pricingengines/` (170 headers, 134 sources)
**Rust crate:** `ql-pricingengines`
**304 files → ~150 Rust modules**

### 16.1 Engine Categories

| Sub-directory | Examples |
|---|---|
| `vanilla/` | `AnalyticEuropeanEngine`, `AnalyticHestonEngine`, `BaroneAdesiWhaleyEngine`, `BjerksundStenslandEngine`, `BinomialEngine`, `FdBlackScholesVanillaEngine`, `MCEuropeanEngine`, `MCAmericanEngine`, `IntegralEngine` |
| `barrier/` | `AnalyticBarrierEngine`, `AnalyticDoubleBarrierEngine`, `FdHestonBarrierEngine`, `MCBarrierEngine` |
| `asian/` | `AnalyticDiscreteGeometricAveragePriceAsianEngine`, `MCDiscreteArithmeticAveragePriceAsianEngine`, `FdBlackScholesAsianEngine` |
| `bond/` | `DiscountingBondEngine`, `TreeCallableFixedRateBondEngine`, `BlackCallableFixedRateBondEngine` |
| `swap/` | `DiscountingSwapEngine`, `TreeSwapEngine` |
| `swaption/` | `BlackSwaptionEngine`, `BachelierSwaptionEngine`, `TreeSwaptionEngine`, `G2SwaptionEngine`, `JamshidianSwaptionEngine`, `FdHullWhiteSwaptionEngine`, `FdG2SwaptionEngine` |
| `capfloor/` | `BlackCapFloorEngine`, `BachelierCapFloorEngine`, `TreeCapFloorEngine`, `AnalyticCapFloorEngine` |
| `credit/` | `MidPointCdsEngine`, `IntegralCdsEngine`, `IsdaCdsEngine` |
| `inflation/` | `YoYInflationCapFloorEngine`, `YoYInflationBachelierCapFloorEngine` |
| `forward/` | `ForwardPerformanceVanillaEngine`, `ReplicatingVarianceSwapEngine` |
| `lookback/` | `AnalyticContinuousFloatingLookbackEngine`, `AnalyticContinuousFixedLookbackEngine`, `AnalyticContinuousPartialFloatingLookbackEngine` |
| `basket/` | `MCEuropeanBasketEngine`, `StulzEngine`, `KirkEngine` |
| `cliquet/` | `AnalyticCliquetEngine`, `AnalyticPerformanceEngine`, `MCPerformanceEngine` |
| `quanto/` | `QuantoEuropeanEngine`, `QuantoForwardVanillaEngine` |
| `exotic/` | `AnalyticHolderExtensibleOptionEngine`, `AnalyticSimpleChooserEngine`, `AnalyticComplexChooserEngine` |
| `futures/` | Futures pricing utilities |

### 16.2 Engine Pattern in Rust

```rust
// The GenericEngine pattern maps directly:
pub struct AnalyticEuropeanEngine {
    process: Arc<GeneralizedBlackScholesProcess>,
}

impl PricingEngine for AnalyticEuropeanEngine {
    type Arguments = VanillaOptionArguments;
    type Results = VanillaOptionResults;

    fn calculate(&self, args: &Self::Arguments) -> Result<Self::Results> {
        // Black-Scholes formula implementation
    }
}
```

---

## 17. Phase 10 — Indexes, Currencies & Quotes

This phase enriches the earlier crates with the more advanced cross-cutting features:

- `InterestRate` struct (compounding, frequency, day counter conversions)
- `TimeSeries<T>` generic (historical fixings)
- Advanced `Money` arithmetic with exchange rate chains
- Index fixing history management
- Integration of indexes with term structures (forecast vs. fixing)

---

## 18. Phase 11 — Experimental Module

**C++ sources:** `ql/experimental/` (263 headers, 158 sources across 26 sub-dirs)
**Rust crate:** `ql-experimental` (feature-gated)

### Sub-modules by size

| Sub-module | Files | What it Contains |
|---|---|---|
| `credit/` | 72 | Synthetic CDO, NTD, Gaussian/Student-t copula models, basket losses |
| `finitedifferences/` | 43 | Extended FDM: local-vol w/ jumps, SABR FDM, etc. |
| `commodities/` | 42 | Energy commodity pricing framework |
| `volatility/` | 38 | SABR smile sections, svi, Zabr interpolation, no-arb SABR |
| `math/` | 34 | Gaussian copula policy, multi-path generator, etc. |
| `exoticoptions/` | 29 | Compound, Chooser, Shout, Extendible, Partial barrier, etc. |
| `barrieroption/` | 16 | Perturbative barrier engine, double barrier |
| `coupons/` | 15 | Range accrual, CMS spread, digital coupons |
| `inflation/` | 16 | Interpolated YoY-quoted cap/floor vol surfaces |
| `variancegamma/` | 13 | Variance gamma model + engines |
| `callablebonds/` | 13 | Callable fixed-rate bond pricing |
| `processes/` | 13 | Extended Ornstein-Uhlenbeck, KLMS, etc. |
| `mcbasket/` | 12 | MC basket options with path-wise Greeks |
| `catbonds/` | 9 | Catastrophe bond framework (BetaRisk, etc.) |
| `basismodels/` | 7 | Tenor basis models |
| `swaptions/` | 7 | Haganirregularswaptionengine, etc. |
| `asian/` | 5 | Analytic continuous geometric Asian engine |
| `lattices/` | 3 | Extended trees (e.g., credit lattice) |
| `models/` | 5 | Extended short-rate models |
| `shortrate/` | 5 | Generalized Hull-White |
| `termstructures/` | 5 | Basis swap rate helpers |
| `varianceoption/` | 5 | Variance option (realized vol derivative) |
| `averageois/` | 4 | Average OIS rate helpers |
| `forward/` | 3 | Forward rate agreement variant |
| `fx/` | 3 | FX forward / composite vol |
| `risk/` | 3 | Sensitivity analysis |

Each sub-module is behind its own Cargo feature flag so users can opt in selectively.

---

## 19. Test-First Strategy

Tests are **not a separate phase** — they are ported **before or alongside** the
implementation in every phase. This is the single most important process decision:

### Why Tests First?

1. **Tests are the specification.** The C++ test suite encodes the exact numerical
   output, edge cases, and behavioral contracts of every QuantLib function. Porting
   them first tells us precisely what "correct" means.
2. **Early error detection.** A bug in Phase 1 (e.g., Observer notification order)
   would silently corrupt every later phase. Tests catch it immediately.
3. **Incremental confidence.** After each phase, `cargo test` gives a green/red
   signal. No guessing.
4. **Refactoring safety.** When making Rust-idiomatic changes to the translated code,
   the ported tests ensure behavior is preserved.

### Workflow per Phase

```
1. Read the C++ test file (e.g., test-suite/dates.cpp)
2. Port test cases to Rust (#[test] functions) → they won't compile yet
3. Implement the minimum types/functions to make the tests compile
4. Run tests → red (logic not yet translated)
5. Translate the implementation from C++
6. Run tests → green
7. Iterate until all tests pass with correct tolerances
```

### Test Placement

- **Unit tests** (`#[cfg(test)] mod tests`) live inside each crate, co-located with
  the module they test. Prefer this for focused, per-function tests.
- **Integration tests** (`crates/ql-*/tests/`) for tests that exercise the crate's
  public API end-to-end — these map 1:1 to QuantLib's `test-suite/*.cpp` files.

### Test File Mapping (Complete)

| C++ Test File | Rust Location | Ported In Phase |
|---|---|---|
| `test-suite/observable.cpp` | `crates/ql-core/tests/test_observable.rs` | **1** |
| `test-suite/errors.cpp` | `crates/ql-core/tests/test_errors.rs` | **1** |
| `test-suite/dates.cpp` | `crates/ql-time/tests/test_dates.rs` | **2** |
| `test-suite/calendars.cpp` | `crates/ql-time/tests/test_calendars.rs` | **2** |
| `test-suite/daycounters.cpp` | `crates/ql-time/tests/test_day_counters.rs` | **2** |
| `test-suite/schedule.cpp` | `crates/ql-time/tests/test_schedule.rs` | **2** |
| `test-suite/matrices.cpp` | `crates/ql-math/tests/test_matrices.rs` | **3** |
| `test-suite/array.cpp` | `crates/ql-math/tests/test_array.rs` | **3** |
| `test-suite/interpolations.cpp` | `crates/ql-math/tests/test_interpolations.rs` | **3** |
| `test-suite/distributions.cpp` | `crates/ql-math/tests/test_distributions.rs` | **3** |
| `test-suite/solvers1d.cpp` | `crates/ql-math/tests/test_solvers.rs` | **3** |
| `test-suite/optimizers.cpp` | `crates/ql-math/tests/test_optimizers.rs` | **3** |
| `test-suite/rngtraits.cpp` | `crates/ql-math/tests/test_rng.rs` | **3** |
| `test-suite/lowdiscrepancysequences.cpp` | `crates/ql-math/tests/test_quasi_rng.rs` | **3** |
| `test-suite/statistics.cpp` | `crates/ql-math/tests/test_statistics.rs` | **3** |
| `test-suite/integrals.cpp` | `crates/ql-math/tests/test_integrals.rs` | **3** |
| `test-suite/currencies.cpp` | `crates/ql-currencies/tests/test_currencies.rs` | **4** |
| `test-suite/quotes.cpp` | `crates/ql-quotes/tests/test_quotes.rs` | **4** |
| `test-suite/termstructures.cpp` | `crates/ql-termstructures/tests/test_term_structures.rs` | **5** |
| `test-suite/piecewiseyieldcurve.cpp` | `crates/ql-termstructures/tests/test_piecewise.rs` | **5** |
| `test-suite/fittedbonddiscountcurve.cpp` | `crates/ql-termstructures/tests/test_fitted_bond.rs` | **5** |
| `test-suite/swaptionvolatilitymatrix.cpp` | `crates/ql-termstructures/tests/test_swaption_vol.rs` | **5** |
| `test-suite/hestonmodel.cpp` | `crates/ql-models/tests/test_heston.rs` | **6** |
| `test-suite/shortratemodels.cpp` | `crates/ql-models/tests/test_short_rate.rs` | **6** |
| `test-suite/marketmodel.cpp` | `crates/ql-models/tests/test_market_model.rs` | **6** |
| `test-suite/fdm.cpp` | `crates/ql-methods/tests/test_fdm.rs` | **7** |
| `test-suite/latticemethods.cpp` (*)  | `crates/ql-methods/tests/test_lattice.rs` | **7** |
| `test-suite/bonds.cpp` | `crates/ql-instruments/tests/test_bonds.rs` | **8** |
| `test-suite/swaps.cpp` | `crates/ql-instruments/tests/test_swaps.rs` | **8** |
| `test-suite/overnightindexedswap.cpp` | `crates/ql-instruments/tests/test_ois.rs` | **8** |
| `test-suite/cashflows.cpp` | `crates/ql-cashflows/tests/test_cashflows.rs` | **8** |
| `test-suite/capfloor.cpp` | `crates/ql-instruments/tests/test_cap_floor.rs` | **8** |
| `test-suite/swaptions.cpp` | `crates/ql-instruments/tests/test_swaptions.rs` | **8** |
| `test-suite/creditdefaultswap.cpp` | `crates/ql-instruments/tests/test_cds.rs` | **8** |
| `test-suite/europeanoption.cpp` | `crates/ql-pricingengines/tests/test_european.rs` | **9** |
| `test-suite/americanoption.cpp` | `crates/ql-pricingengines/tests/test_american.rs` | **9** |
| `test-suite/asianoptions.cpp` | `crates/ql-pricingengines/tests/test_asian.rs` | **9** |
| `test-suite/barrieroption.cpp` | `crates/ql-pricingengines/tests/test_barrier.rs` | **9** |
| `test-suite/lookbackoptions.cpp` | `crates/ql-pricingengines/tests/test_lookback.rs` | **9** |
| `test-suite/basketoption.cpp` | `crates/ql-pricingengines/tests/test_basket.rs` | **9** |
| `test-suite/cliquetoption.cpp` | `crates/ql-pricingengines/tests/test_cliquet.rs` | **9** |
| `test-suite/quantooption.cpp` | `crates/ql-pricingengines/tests/test_quanto.rs` | **9** |
| `test-suite/forwardoption.cpp` | `crates/ql-pricingengines/tests/test_forward.rs` | **9** |
| … | … | … |

### Numerical Tolerance Convention

| Comparison Type | Default Tolerance | Notes |
|---|---|---|
| Exact integer / date | `assert_eq!` | No tolerance |
| Price / NPV | `1e-8` | Matches most C++ test tolerances |
| Greeks (delta, gamma, vega) | `1e-4` to `1e-6` | Finite-difference Greeks are noisier |
| Yield / rate | `1e-10` | Rate-sensitive |
| Monte Carlo | `1e-2` to `1e-3` | Statistical; use same seed as C++ |
| Matrix decomposition | `1e-12` | Near machine precision |

Always use the **same tolerance as the C++ test** — grep for `tolerance` or `eps` in
the original test file.

---

## 20. C++ Pattern → Rust Idiom Reference

This is a quick-reference table for the translator. For every C++ pattern encountered,
apply the corresponding Rust idiom.

| # | C++ Pattern | Rust Idiom | Example |
|---|---|---|---|
| 1 | Abstract base class with pure virtual | `trait` | `class PricingEngine` → `trait PricingEngine` |
| 2 | Concrete class | `struct` + `impl Trait` | `class VanillaOption` → `struct VanillaOption` |
| 3 | Multiple inheritance | Multiple trait bounds | `class LazyObject: public Observable, public Observer` → `trait LazyObject: Observable + Observer` |
| 4 | CRTP | Generic + associated type | `template<class T> class CuriouslyRecurring` → `trait HasDerived { type Derived; }` |
| 5 | Virtual dispatch | `dyn Trait` or enum | `shared_ptr<Instrument>` → `Box<dyn Instrument>` |
| 6 | Pimpl / Bridge | Enum or `Box<dyn Impl>` | `Calendar`'s `shared_ptr<Calendar::Impl>` → `enum Calendar { ... }` |
| 7 | `mutable` + `const` method | `Cell<T>` / `RefCell<T>` + `&self` | `mutable Real NPV_` → `Cell<Option<f64>>` |
| 8 | `shared_ptr<T>` | `Arc<T>` / `Rc<T>` | See §3.3 |
| 9 | `unique_ptr<T>` | `Box<T>` | |
| 10 | Raw pointer (`T*`) | `&T` / `&mut T` / `Weak<T>` | Observer back-refs → `Weak` |
| 11 | `optional<T>` | `Option<T>` | |
| 12 | `pair<A,B>` | `(A, B)` | |
| 13 | `vector<T>` | `Vec<T>` | |
| 14 | `map<K,V>` | `BTreeMap<K,V>` | Ordered maps for stability |
| 15 | Exception | `Result<T, QuantLibError>` | |
| 16 | `QL_REQUIRE` | `ql_require!` → `Result` | |
| 17 | `static` local (lazily initialized) | `LazyLock<T>` / `thread_local!` | |
| 18 | Namespace | Module | `namespace QuantLib` → `mod quantlib` |
| 19 | `#define` constant | `const` or `const fn` | |
| 20 | `typedef` | `type Alias = ...;` | |
| 21 | `enum` | `#[derive] enum` | |
| 22 | `operator<<` (ostream) | `impl Display` | |
| 23 | `operator==`, `<`, etc. | `impl PartialEq, PartialOrd` | |
| 24 | `operator+`, `-`, `*`, `/` | `impl Add, Sub, Mul, Div` | |
| 25 | Copy constructor | `impl Clone` (+`Copy` for small types) | |
| 26 | Destructor with side effects | `impl Drop` | |
| 27 | `friend` function | Public free function in same module | |
| 28 | Nested class | Nested struct or separate file | |
| 29 | Template class | Generic struct `<T: Bound>` | |
| 30 | Template specialization | Separate `impl` block or trait specialization | |
| 31 | `#ifdef` feature toggle | `#[cfg(feature = "...")]` | |
| 32 | Header guard | (automatic in Rust — no equivalent needed) | |

---

## 21. Dependency Strategy

QuantLib's core value is its **financial** logic — instruments, pricing engines, term
structures, calibration, and models. Its math primitives (matrix ops, distributions,
RNG) are standard numerical algorithms that mature Rust crates already implement with
SIMD optimization, extensive testing, and active maintenance. Reimplementing these from
scratch would be ~15,000 lines of undifferentiated work with zero financial value.

**Principle:** Use the best existing Rust crate for every non-financial primitive.
Translate the financial logic that sits *on top* of those primitives.

### 21.1 External Crate Dependencies

| Crate | Replaces (C++) | Where Used | Why |
|---|---|---|---|
| **`nalgebra`** | `ql/math/matrix.hpp`, `ql/math/array.hpp`, `ql/math/matrixutilities/` | `ql-math` | Battle-tested, SIMD-optimized. Cholesky, SVD, QR, eigenvalue decomposition, LU — all built in. ~5M downloads. |
| **`nalgebra` `DVector`/`DMatrix`** | `Array`, `Matrix` | `ql-math` | Dynamic-size vectors and matrices with full operator overloading. Direct replacement for QuantLib's `Array` and `Matrix`. |
| **`statrs`** | `ql/math/distributions/` | `ql-math` | Normal, chi-square, gamma, Student-t, Poisson, binomial distributions with CDF, PDF, inverse CDF. Matches the same textbook algorithms. |
| **`rand`** + **`rand_distr`** | `ql/math/randomnumbers/mersenne_twister.hpp` | `ql-math` | MT19937 with identical PRNG output. Standard Rust RNG ecosystem. |
| **`sobol_burley`** or **`quasirandom`** | `ql/math/randomnumbers/sobol.hpp`, `halton.hpp` | `ql-math` | Sobol/Halton quasi-random sequences. If direction numbers differ from QuantLib, provide a thin adapter (see §21.3). |
| **`num-traits`** | — | `ql-math` | `Float`, `Zero`, `One`, `NumCast` trait bounds for generic numeric code. |
| **`thiserror`** | `ql/errors.hpp` | `ql-core` | Derive macros for `QuantLibError` enum. |
| **`serde`** (optional) | — | All crates | `Serialize`/`Deserialize` behind `feature = "serde"`. |
| **`chrono`** (optional) | — | `ql-time` | `From<NaiveDate>` / `Into<NaiveDate>` conversions only. Internal `Date` stays serial-number based. |
| **`approx`** | — | All tests | `assert_abs_diff_eq!` and `assert_relative_eq!` for float comparisons. |
| **`criterion`** | — | `benches/` | Statistical microbenchmarks. |
| **`proptest`** | — | Tests | Property-based testing for edge cases. |

### 21.2 Thin Wrapper Strategy

We do **not** expose `nalgebra`/`statrs`/`rand` types directly in our public API.
Instead, we define our own types that wrap or delegate to them:

```rust
// crates/ql-math/src/array.rs
use nalgebra::DVector;

/// 1-D array of reals — wraps nalgebra::DVector<f64>.
/// Provides the same API surface as C++ QuantLib's Array.
#[derive(Clone, Debug, PartialEq)]
pub struct Array(pub(crate) DVector<f64>);

impl Array {
    pub fn new(size: usize, value: f64) -> Self {
        Self(DVector::from_element(size, value))
    }
    pub fn dot(&self, other: &Self) -> f64 {
        self.0.dot(&other.0)
    }
    // ... operator overloads delegate to DVector
}

impl From<DVector<f64>> for Array {
    fn from(v: DVector<f64>) -> Self { Self(v) }
}
impl From<Array> for DVector<f64> {
    fn from(a: Array) -> Self { a.0 }
}
```

This gives us:
1. **Stable public API** — our types don't break if nalgebra bumps a major version.
2. **QuantLib-compatible names** — `Array`, `Matrix`, not `DVector`, `DMatrix`.
3. **Zero-cost** — the wrappers are newtypes; they compile away.
4. **Interop** — users who want raw `nalgebra` types get `From`/`Into` for free.

### 21.3 What We Still Implement Ourselves

Some QuantLib math has no off-the-shelf Rust crate equivalent:

| Component | Why Custom |
|---|---|
| **Interpolation** (24 schemes) | QuantLib's interpolation API is tightly coupled to its term-structure framework. No Rust crate matches the exact `Interpolation` trait surface (evaluate, primitive, derivative, second derivative, update). |
| **1D Root Solvers** (Brent, bisection, etc.) | Simple algorithms (~50 LOC each) that are trivial to translate and must match QuantLib's exact iteration/convergence behavior. |
| **Optimization** (L-M, BFGS, simplex, diff. evolution, etc.) | QuantLib's optimizer API (`CostFunction`, `Constraint`, `EndCriteria`) is deeply woven into calibration. The `argmin` crate exists but has a different API shape — wrapping it would be more work than translating. |
| **Copulas** | Niche; no mature Rust crate. Small code (~8 files). |
| **ODE solvers** (Adaptive Runge-Kutta) | Tiny (~3 files). |
| **Sobol direction numbers** | If QuantLib uses specific Joe-Kuo direction numbers that differ from available Rust crates, we ship a compatible table. Wrap the generation logic around `rand`'s API. |

### 21.4 Crate Version Pinning

Pin major versions in `Cargo.toml` to avoid surprise breaks:

```toml
[dependencies]
nalgebra = "0.33"     # or latest stable at project start
statrs = "0.17"
rand = "0.8"
rand_distr = "0.4"
num-traits = "0.2"
thiserror = "2"
```

Run `cargo deny check` in CI to audit for security advisories and license conflicts.

---

## 22. Verification & Quality Gates

### 22.1 Per-Phase Acceptance Criteria

Each phase must satisfy **all** of the following before moving on:

1. **Tests ported first** — The corresponding C++ test-suite files are translated to
   Rust *before* the implementation is considered complete (see §19).
2. **Compiles cleanly** — `cargo build` with zero warnings.
3. **All tests pass** — `cargo nextest run` green, including the newly ported tests.
4. **Clippy clean** — `cargo clippy -- -D warnings`.
5. **Formatted** — `cargo fmt --check` passes.
6. **Numerical equivalence** — For every numerical function, at least one test
   compares Rust output to the known C++ QuantLib output with documented tolerance.
7. **Coverage** — `cargo llvm-cov` reports ≥80% line coverage on the new crate.
8. **No unsafe** — Unless absolutely necessary (SIMD, FFI), with documented rationale.

### 22.2 Continuous Verification Against C++ Reference

For critical numerical paths, maintain a small C++ harness (compiled via `cc` crate in
`build.rs`) that calls the original QuantLib function and compares the output:

```rust
#[test]
fn test_black_scholes_matches_cpp() {
    let rust_result = analytic_european_engine::calculate(...);
    let cpp_result = unsafe { ffi::ql_black_scholes_price(...) };
    assert!((rust_result - cpp_result).abs() < 1e-12);
}
```

### 22.3 Benchmarks

Every performance-critical path gets a `criterion` benchmark:
- Matrix operations (multiply, Cholesky, SVD)
- Black-Scholes pricing
- Monte Carlo simulation convergence
- Yield curve bootstrapping
- FDM solver step

---

## 23. Risk Register

| # | Risk | Impact | Mitigation |
|---|---|---|---|
| 1 | Observer pattern with interior mutability causes borrow checker pain | High — it's QuantLib's most pervasive pattern | Prototype the `Observable`/`Observer`/`Handle` system in Phase 1 before building anything on top. Get it right once. |
| 2 | C++ template metaprogramming doesn't map cleanly to Rust generics | Medium — affects `PiecewiseYieldCurve`, `GenericEngine`, interpolation factories | Use trait objects + builders where templates would create unmaintainable generic bounds in Rust. |
| 3 | Mutable state + `const` methods everywhere (LazyObject) | Medium — Rust's `RefCell<T>` is less ergonomic than C++ `mutable` | Establish a clear `Cell`/`RefCell` convention in Phase 1; apply uniformly. |
| 4 | Numerical precision differences between C++ `double` and Rust `f64` | Low — both are IEEE 754 double | Test with documented tolerances. Be careful with operation ordering. |
| 5 | Market Model framework is enormous (~160 files) and deeply recursive | High — it's the largest single sub-system | Translate it last; it's largely self-contained. |
| 6 | Thread safety: C++ has optional thread-safe observer; Rust needs to decide | Medium | Feature flag: `thread-safe-observers` swaps Rc→Arc, RefCell→RwLock. |
| 7 | Experimental module is large (421 files) and may depend on unstable C++ APIs | Low — it's isolated | Feature-gate the entire `ql-experimental` crate. |
| 8 | Maintaining parity as QuantLib upstream evolves | Medium — QuantLib releases ~quarterly | Pin to a specific QuantLib git commit. Update periodically with diff-based patches. |

---

## Appendix A: Quick Start for Contributors

```bash
# Enter the dev shell
nix develop

# Build everything
cargo build --workspace

# Run all tests
cargo nextest run --workspace

# Check coverage
cargo llvm-cov --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Count translation progress
tokei crates/
```

## Appendix B: File Naming Conventions

| Aspect | Convention | Example |
|---|---|---|
| Crate name | `ql-{module}` | `ql-core`, `ql-time`, `ql-math` |
| Rust file | `snake_case.rs` | `piecewise_yield_curve.rs` |
| Struct name | `PascalCase` | `PiecewiseYieldCurve` |
| Trait name | `PascalCase` | `YieldTermStructure` |
| Function name | `snake_case` | `year_fraction()` |
| Constant | `SCREAMING_SNAKE` | `M_SQRT2` → `SQRT_2` |
| Feature flag | `kebab-case` | `thread-safe-observers` |
| Test function | `test_snake_case` | `test_flat_forward_zero_rate` |

## Appendix C: Commit Message Convention

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
