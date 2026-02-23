{
  description = ''
    Development environment for translating QuantLib (C++) → Rust.

    QuantLib is a large quantitative-finance library (~500 k LOC of C++17) that
    depends primarily on Boost.  This flake provisions:

      • The full C++ reference stack (Boost, CMake, Clang tooling) so you can
        build and read QuantLib in-place while writing the Rust equivalent.
      • A pinned Rust stable toolchain with every component needed for serious
        library development (rust-analyzer, clippy, rustfmt, rust-src, llvm-tools).
      • A separate `nightly` shell that adds Miri (UB detector), cargo-fuzz and
        other unstable tools — invaluable when verifying numeric correctness.
      • bindgen CLI so you can auto-generate skeleton FFI bindings from QuantLib
        headers as a structural reference or for thin-wrapper tests.
      • A curated set of cargo extensions (nextest, criterion, expand, watch, …).
      • Profiling / correctness tooling (valgrind, heaptrack, flamegraph, hyperfine).
  '';

  # ---------------------------------------------------------------------------
  # Inputs
  # ---------------------------------------------------------------------------
  inputs = {
    # Track unstable so we get recent Boost, LLVM, and cargo extension versions.
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # rust-overlay gives us reproducible, component-aware Rust toolchain pins
    # without relying on rustup at runtime.
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs"; # stay in sync with our nixpkgs
    };

    # flake-utils removes the per-system boilerplate.
    flake-utils.url = "github:numtide/flake-utils";
  };

  # ---------------------------------------------------------------------------
  # Outputs
  # ---------------------------------------------------------------------------
  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Inject the rust-overlay so `pkgs.rust-bin` is available.
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # ── Rust toolchains ──────────────────────────────────────────────────

        # Stable toolchain: the workhorse.  We request rust-src so that
        # rust-analyzer can provide full std type inference, and llvm-tools for
        # coverage instrumentation with cargo-llvm-cov.
        rustStable = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"          # jump-to-definition into std; required by rust-analyzer
            "rust-analyzer"     # LSP (the nix-provided binary, no separate install needed)
            "clippy"            # linter
            "rustfmt"           # formatter
            "llvm-tools-preview" # needed by cargo-llvm-cov and cargo-binutils
          ];
        };

        # Nightly toolchain: used in the `nightly` shell for Miri, fuzzing, and
        # unstable Rust features that may ease translation (e.g. f128, portable-simd).
        rustNightly = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
            "miri"              # undefined-behaviour detector; great for numeric code
            "llvm-tools-preview"
          ];
        };

        # ── C++ reference stack ──────────────────────────────────────────────
        #
        # QuantLib's only mandatory runtime dep is Boost (≥ 1.58 for C++17).
        # We also pull in the full Clang toolchain so you can:
        #   • build QuantLib locally for interactive comparison / testing
        #   • run clang-tidy / clangd on the C++ source while studying it
        #   • give bindgen a libclang to parse headers with
        cxxStack = with pkgs; [
          # --- compiler & build system ---
          gcc                   # GCC (alternative to clang for building QuantLib)
          clang                 # Clang compiler
          clang-tools           # clangd (C++ LSP), clang-format, clang-tidy, clang-check
          cmake                 # QuantLib uses CMake ≥ 3.15
          ninja                 # faster CMake generator backend
          pkg-config

          # --- Boost ------------------------------------------------------------------
          # QuantLib requires: Boost.signals2, Boost.unit_test, Boost.interprocess,
          # plus Boost.math, Boost.bind, Boost.iterator, etc. throughout the codebase.
          # boost.dev gives us headers; boost gives us compiled libs.
          boost.dev
          boost

          # --- libclang (required by `bindgen`) ---
          # bindgen calls into libclang to parse C++ headers.  LIBCLANG_PATH must
          # point at this library's lib/ directory (set in shellHook below).
          llvmPackages.libclang
          llvmPackages.libclang.lib

          # --- documentation generation ---
          # QuantLib ships Doxygen config; generating docs locally is the fastest
          # way to navigate the 500k-LOC codebase while translating.
          doxygen
          graphviz  # Doxygen calls dot for class/collaboration diagrams
        ];

        # ── Rust ecosystem tools ─────────────────────────────────────────────
        #
        # These extend cargo with quality-of-life and analysis commands that are
        # particularly useful during a large translation project.
        cargoExtensions = with pkgs; [
          # Dependency management
          cargo-edit            # cargo add / rm / upgrade (like npm for Rust)
          cargo-deny            # audit licenses, duplicates, security advisories
          cargo-udeps           # find unused dependencies (keeps crate slim)

          # Development workflow
          cargo-watch           # `cargo watch -x test` — rebuild on every save
          cargo-expand          # show macro-expanded code (vital when writing macros
                                #   that mirror C++ template machinery)
          cargo-nextest         # faster, better-structured test runner

          # Profiling & benchmarking (to verify you match C++ performance)
          cargo-criterion       # statistical microbenchmarks (wraps Criterion.rs)
          cargo-flamegraph      # generate flamegraphs with `cargo flamegraph`
          cargo-llvm-cov        # line/branch coverage reports

          # Correctness
          cargo-semver-checks   # ensure public API doesn't regress between versions

          # bindgen CLI — generate Rust FFI bindings directly from QuantLib headers.
          # Useful as a structural sanity-check or to write thin C-wrapper tests
          # that verify your pure-Rust impl matches C++ output.
          rust-bindgen

          # cargo-binutils (nm, objdump, size …) for inspecting compiled artefacts
          cargo-binutils
        ];

        # ── Native profiling & correctness tools ────────────────────────────
        nativeTools = with pkgs; [
          # Memory / correctness
          valgrind              # Memcheck, Callgrind — run against C++ reference builds
          heaptrack             # heap profiler (compare memory layout C++ vs Rust)

          # Profiling
          perf                  # Linux perf (used by cargo-flamegraph)
          hyperfine             # shell-command benchmarking (compare binaries)

          # Code navigation / productivity
          ripgrep               # fast `rg` — search QuantLib source while translating
          fd                    # fast `find` replacement
          tokei                 # count lines of code; track how much is translated
          just                  # justfile task runner (like make but nicer)
          jq                    # parse/transform JSON (test output, benchmarks)
          git                   # obviously
          gnumake               # some QuantLib examples use plain Makefiles
        ];

        # ── Helper: build a dev shell with a given Rust toolchain ────────────
        mkQuantlibShell = { rustToolchain, shellName, extraPackages ? [ ] }:
          pkgs.mkShell {
            name = shellName;

            packages =
              [ rustToolchain ]
              ++ cxxStack
              ++ cargoExtensions
              ++ nativeTools
              ++ extraPackages;

            # ── Environment variables ─────────────────────────────────────
            env = {
              # bindgen needs to locate libclang at compile time.
              LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

              # Make cmake / bindgen find Boost headers without extra flags.
              BOOST_ROOT = "${pkgs.boost.dev}";
              BOOST_INCLUDEDIR = "${pkgs.boost.dev}/include";
              BOOST_LIBRARYDIR = "${pkgs.boost}/lib";

              # Rich backtraces during development.
              RUST_BACKTRACE = "full";

              # Default log level for env_logger / tracing.
              RUST_LOG = "debug";

              # Ensure clang is used for C/C++ compilation invoked by build.rs
              # scripts (common when cargo crates compile C++ via the `cc` crate).
              CC  = "clang";
              CXX = "clang++";
            };

            shellHook = ''
              # ------------------------------------------------------------------
              # Pretty banner
              # ------------------------------------------------------------------
              echo ""
              echo "  ╔══════════════════════════════════════════════════════╗"
              echo "  ║       QuantLib  ──────→  Rust  Translation Env      ║"
              echo "  ╚══════════════════════════════════════════════════════╝"
              echo ""
              echo "  Shell     : ${shellName}"
              echo "  Rust      : $(rustc --version)"
              echo "  Cargo     : $(cargo --version)"
              echo "  Clang     : $(clang --version | head -1)"
              echo "  Boost     : ${pkgs.boost.version}"
              echo "  bindgen   : $(bindgen --version)"
              echo ""
              echo "  Useful commands:"
              echo "    bindgen <header.h> --output <out.rs>   generate FFI scaffold"
              echo "    cargo watch -x 'test'                  rebuild on save"
              echo "    cargo nextest run                      parallel test runner"
              echo "    cargo expand <module>                  inspect macro output"
              echo "    cargo flamegraph                       generate flamegraph"
              echo "    cargo llvm-cov                         coverage report"
              echo "    tokei                                  count translated LOC"
              echo "    just                                   run project tasks"
              echo ""
            '';
          };

      in
      {
        # ── devShells ─────────────────────────────────────────────────────────

        devShells = {
          # `nix develop` — day-to-day translation work on stable Rust.
          default = mkQuantlibShell {
            rustToolchain = rustStable;
            shellName     = "quantlib-rs-stable";
          };

          # `nix develop .#nightly` — deep correctness work.
          #
          # Adds:
          #   • Miri — run `cargo miri test` to catch undefined behaviour in
          #     unsafe blocks that you may need when implementing numeric kernels.
          #   • cargo-fuzz — fuzz-test translated routines against C++ reference.
          nightly = mkQuantlibShell {
            rustToolchain = rustNightly;
            shellName     = "quantlib-rs-nightly";
            extraPackages = with pkgs; [
              cargo-fuzz   # libFuzzer-based fuzzing; compare Rust output vs QuantLib
              aflplusplus  # AFL++ — alternative coverage-guided fuzzer
            ];
          };
        };
      }
    );
}
