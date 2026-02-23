# quantlib-rs

A complete Rust translation of the [QuantLib](https://www.quantlib.org/) C++
quantitative finance library.

> **Status**: Early scaffolding — workspace structure and crate stubs in place.
> Implementation of Phase 1 (core types, date, calendar) is underway.

## Goals

- **1:1 functional parity** with QuantLib C++ (v1.x)
- Idiomatic Rust: `Result` error handling, traits instead of OOP hierarchies,
  zero-cost abstractions
- Test-first — C++ test suite ported alongside each module
- Leverage proven Rust crates (nalgebra, statrs, rand) where appropriate

## Workspace layout

| Crate | Description |
|-------|-------------|
| `ql-core` | Type aliases, error types, Observer/Observable, Handle, Settings |
| `ql-time` | Date, Calendar, DayCounter, Schedule, BusinessDayConvention |
| `ql-math` | Interpolation, solvers, optimisation, Array/Matrix newtypes, RNG |
| `ql-currencies` | Currency and exchange-rate definitions |
| `ql-quotes` | Market quotes and the `Quote` trait |
| `ql-indexes` | Interest-rate, inflation, and equity indexes |
| `ql-termstructures` | Yield curves, vol surfaces, default-probability curves |
| `ql-processes` | Stochastic process definitions |
| `ql-models` | Short-rate, equity, and credit models |
| `ql-methods` | Lattice, finite-difference, and Monte Carlo frameworks |
| `ql-cashflows` | Cash flows, coupons (fixed, floating, CMS), legs |
| `ql-instruments` | Bonds, swaps, options, caps/floors, swaptions, … |
| `ql-pricingengines` | Analytic, lattice, FDM, and MC pricing engines |
| `ql-experimental` | Experimental / unstable extensions |
| `ql-legacy` | Deprecated modules preserved for completeness |
| **`quantlib`** | Façade crate — re-exports everything for end users |

## Getting started

```bash
# Enter the Nix dev shell (provides Rust toolchain + C++ reference stack)
nix develop

# Build all crates
cargo build --workspace

# Run all tests
cargo nextest run --workspace

# Run clippy
cargo clippy --workspace --all-targets -- -D warnings
```

## Contributing

See [plan.md](plan.md) for the full translation plan, phase ordering,
and design-pattern mapping.

## License

BSD 3-Clause — see [LICENSE](LICENSE).

---

*quantlib-rs is not affiliated with or endorsed by the QuantLib project.*
