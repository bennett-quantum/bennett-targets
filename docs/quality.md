# Validator quality measurements

Round 1 uses only the ISA checkout selected by `gate.toml` through
`.tmp/gate/cache/bennett-isa-spec@ecaaac060c44/bennett-isa-spec`.
Tests and the consumer harness fail when the selected cache is absent.

The five round-2 regressions pass (import precision, resolved placement classes,
native model admission, decimal grammar and native operand arity):
`timeout 150s cargo test --offline --manifest-path validator/Cargo.toml -j3 --test review_round2 -- --test-threads=4`
(before consolidation; now `--test integration review_round2::`). No outstanding
round-2 P0/P1 was reproduced.

The first pinned default run passed all 50 tests. After merging eleven integration
binaries into one and setting `[profile.test] debug = "line-tables-only"`,
`/usr/bin/time -f 'default wall=%e seconds' timeout 150s cargo test --offline --manifest-path validator/Cargo.toml -j3 -- --test-threads=4`
took **3.91 seconds**, including compilation (test execution 0.08 seconds).
There is no slow or promised test to exclude in this repo.

Baseline LLVM line coverage: **89.06% (1645/1847 lines)**, measured by
`timeout 180s cargo llvm-cov --offline --manifest-path validator/Cargo.toml -j3 --json --output-path .tmp/quality-targets/coverage-baseline.json -- --test-threads=4`.
The five least-covered modules were main.rs (67.35%), model.rs (68.08%),
runtime.rs (91.84%), importers.rs (91.95%) and target.rs (92.76%).
The follow-up pass exercises CLI import/error paths, native matrix arithmetic,
runtime diagnostics and aggregation, external importer samples, and malformed
schema fields. Host-limit branches are bounded errors rather than enormous test
allocations. Final measurements and the resulting coverage are recorded below.

All commands use DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS and BROWSER
unset; TMPDIR=$PWD/.tmp/quality-targets/tmp and
CARGO_TARGET_DIR=$PWD/.tmp/quality-targets/target. Create TMPDIR before running.

## Final round-1 verification

Measured 2026-11-06T06:47:39Z. LLVM executable-line coverage is **93.55%
(1797/1921 lines)** using the same pinned environment:

```sh
timeout 180s cargo llvm-cov --offline --manifest-path validator/Cargo.toml -j3 --json --output-path .tmp/quality-targets/coverage-final.json -- --test-threads=4
```

This is the tool's line-coverage total, including the in-file native-model unit
tests; no threshold or claim of exhaustive semantics is inferred. All 65 Rust
tests passed during coverage (2 unit, 63 integration).

The final default run took **4.83 seconds**, including compilation; integration
execution took 0.37 seconds. It passed all 65 tests with none ignored:

```sh
/usr/bin/time -f 'default wall=%e seconds' timeout 150s cargo test --offline --manifest-path validator/Cargo.toml -j3 -- --test-threads=4
timeout 30s cargo fmt --manifest-path validator/Cargo.toml -- --check
timeout 120s cargo clippy --offline --manifest-path validator/Cargo.toml -j3 --all-targets -- -D warnings
timeout 30s python3 -m unittest discover -s scripts -p 'test_*.py' -v
```

Formatting and strict Clippy passed. All five Python tests passed; they cover
missing/malformed pins, refusing a live sibling fallback, and stable consumer
report diagnostics. Python source syntax and gate TOML were also parsed. The
pinned full consumer gate was not run, and docs/verification.json remains
historical. The orchestrator runs that gate and arranges independent review.

| Baseline least-covered module | Final coverage | Follow-up evidence |
| --- | --- | --- |
| main.rs | 91.84% | CLI success paths for both real imports; malformed queries and arguments produce JSON errors |
| model.rs | 88.45% | Independent basis permutations/phases for X, S, S-adjoint, CZ and SWAP; exact complex arithmetic and division-by-zero |
| runtime.rs | 94.12% | Four red-first panic/diagnostic regressions; 32-schedule discrete occupancy oracle with borrowing; heterogeneous exact import factors |
| importers.rs | 91.95% | Actual Qualtran QFT counts and every zero-default flag; Microsoft report projection, gate times and preserved epsilon |
| target.rs | 95.09% | Missing/invalid classes, pointer widths, objective and capability fields |

The largest production functions are schema (220 lines), aggregate (191),
expand (150), and realisation_model (134), measured from formatted declarations
through their matching closing indentation. No production function exceeds
300 lines. Runtime's aggregate lookup helper was extracted alongside its error
changes; no standalone function-splitting campaign was necessary.

Provenance now compares every numeric leaf to a sidecar ledger by JSON pointer
and exact rational value. The checker failed on the old ISA label and again on
superconducting two-wire decimal drift before those were corrected. Mutation
checks detect swapped values, extra numbers and stale paths. Routing executes
all sixteen basis states, the Toffoli check uses 128 machine epsilons, and the
published E-import witness calls the actual conversion API.

Runtime red-first checks reproduced both input-derived unwrap panics, an unsafe
wire-subscript slice and the missing-class vertex-limit fallback. The public
Target document is mutable, so runtime callers can reach these paths even after
a successful load. They now return errors with field/rule context. Defaults that
remain are the explicit optional-field/default-rate rules of ISA §8.2.

The existing worktree index already excludes the Beads locks/backups named in
the audit. A read-only parse of this worktree's index found only Cargo.lock among
tracked lock/backup/cache candidates; Cargo.lock is the dependency lockfile and
is intentionally retained. Added ignore rules cover runtime locks, .darc,
Python caches and logs. There are no actual issue/job identifiers in Rust
production source or maintained Python implementation strings. Consumer reports
use stable capability keys, with the open-issue mapping retained in HANDOFF.md.

Remaining scope and evidence limits:

- b-targets-38r / b-targets-7dv / b-targets-dt2 remain consumer limitations at the
  existing pins; this pass does not claim a new consumer agreement. The missing
  successful 2D-grid timing baseline was added to the harness, but its conditional
  consumer path needs the merge gate. The other b-targets-5gr nits were addressed;
  its final consumer-dependent verification remains open.
- b-targets-gyp and b-targets-1wr retain the complete QEC/hierarchy/symbolic and
  native-model encoding scope. Microsoft output is explicitly projected to its
  supported hardware parameters; the full report is rejected and preserved.
- The ISA pin predates bennett.native-model/1 (K86). Round-2 native-model tests
  enforce the accepted review contract, while the pinned suite cannot supply a
  K86 oracle. This limitation is filed for the orchestrator to repin once the
  intended published revision and conformance vectors are selected.
