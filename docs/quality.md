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
expand (152), and realisation_model (134), measured from formatted declarations
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

## Push quality ratchet

Measured 2026-11-06T07:01:46Z with `python3 scripts/quality_ratchet.py --measure`.
The single `quality-baseline` gate check runs `python3 scripts/quality_ratchet.py`.
Only Git-tracked files are measured, including Rust/Python tests and the metric
script itself; untracked build products are excluded. The script uses only the
Python standard library and Git. It performs the structural measurements and
both identifier/litter greps; there is no separate metrics tool or threshold.

Each file records its entire function-length distribution in descending order
(declaration through closing brace/end line, including blank/comment lines).
The gate compares every rank, padding a missing rank with zero, so no longer
function or added function escapes review. New files start with a zero baseline.
A deliberate new function/file therefore needs an explicit, reviewed baseline
update using `--measure`; the check never rewrites its baseline. Improvements in
one file cannot offset deterioration in another.

`complexity` is a reproducible local cognitive-complexity proxy, not a claim of
Sonar/AST equivalence across languages. Rust's lexer removes comments (including
nested block comments) and string/character literals. Within each function,
`if`, `for`, `while`, `loop` and `match` cost one plus lexical brace nesting;
`&&`, `||` and `?` cost one each. Nested functions are measured separately, never
double-counted in the mass. Rust syntax validity remains the compiler's job.
Python uses its AST: conditionals, loops, exception handlers, matches and
comprehensions cost one plus control nesting; Boolean links cost one. Module
control flow is included, and nested functions reset nesting. This proxy does
not account for every language construct or closure as an independent function.

`wrappers` counts single call/return forwarding bodies with simple positional
arguments (Rust also permits `return`, `?` and a trailing semicolon). This is a
syntactic trivial-wrapper count, not a purity or API-necessity judgment. It
ratchets separately so shorter forwarding functions cannot buy complexity relief.
All three per-file metrics must stay at or below their recorded values.

The litter grep covers tracked `__pycache__`, `.pyc`, `.log`, `.darc`, `LOCK`,
`.lock`, `*.lock`, `PROGRESS*` and `LAST-RUN*`; reproducible `Cargo.lock` is exempt.
The only existing hit, `docs/PROGRESS.md`, is outside this job's owned paths;
b-targets-u8u tracks retirement. No new litter path is allowed. The process grep
covers actual ecosystem bead IDs, `job/…`, quality-job names and numbered
shift/round/session labels in production source, comments and target/importer
JSON (also filenames). Test files and Rust's trailing `#[cfg(test)]` modules
are exempt. Historical documentation/verification reports are outside this
product scan. The current process-identifier count is zero.

Red-first evidence: the new tests failed before the script existed. Scratch
repositories under TMPDIR then demonstrate a longer function failing with exit
1, equal-length nested branches increasing mass, a shorter wrapper still failing,
new-file/litter/identifier regressions failing, and structural improvement passing.
They also exercise literal/comment exclusion and Python module control flow:
`timeout 30s python3 -m unittest discover -s scripts -p 'test_quality_ratchet.py' -v`.
No deliberately worse file was written into the working tree.

<!-- quality-baseline -->
```json
{
  "files": {
    "scripts/capture_importer_samples.py": {"complexity": 3, "lengths": [38], "wrappers": 0},
    "scripts/consumer_report.py": {"complexity": 7, "lengths": [9], "wrappers": 0},
    "scripts/gate_pin.py": {"complexity": 7, "lengths": [12], "wrappers": 0},
    "scripts/quality_ratchet.py": {"complexity": 114, "lengths": [40, 22, 20, 18, 15, 13, 8], "wrappers": 0},
    "scripts/test_consumer_report.py": {"complexity": 1, "lengths": [5, 3], "wrappers": 0},
    "scripts/test_gate_pin.py": {"complexity": 2, "lengths": [8, 8, 8], "wrappers": 0},
    "scripts/test_quality_ratchet.py": {"complexity": 1, "lengths": [13, 11, 11, 8, 8, 8, 7, 7, 7, 6, 5, 5], "wrappers": 0},
    "scripts/verify.py": {"complexity": 103, "lengths": [24, 14, 13, 11, 9, 8, 4, 3, 2, 2, 2], "wrappers": 0},
    "validator/src/expr.rs": {"complexity": 88, "lengths": [81, 49, 23, 16, 12, 9, 8, 8, 7, 6], "wrappers": 0},
    "validator/src/importers.rs": {"complexity": 89, "lengths": [78, 56, 46, 26, 21, 8, 8, 7, 4], "wrappers": 0},
    "validator/src/json.rs": {"complexity": 73, "lengths": [107, 66, 36, 14, 10, 3], "wrappers": 0},
    "validator/src/lib.rs": {"complexity": 2, "lengths": [6, 3], "wrappers": 0},
    "validator/src/main.rs": {"complexity": 28, "lengths": [49, 9], "wrappers": 0},
    "validator/src/model.rs": {"complexity": 108, "lengths": [134, 51, 37, 35, 27, 24, 7, 3], "wrappers": 0},
    "validator/src/query.rs": {"complexity": 15, "lengths": [12, 12, 11], "wrappers": 0},
    "validator/src/runtime.rs": {"complexity": 270, "lengths": [191, 152, 23, 10, 8, 8, 4, 4], "wrappers": 0},
    "validator/src/target.rs": {"complexity": 265, "lengths": [220, 55, 21, 17, 16, 14, 12, 12, 11, 11, 10, 8, 7, 7, 7, 7, 7, 6, 6, 3, 3, 3, 3], "wrappers": 1},
    "validator/src/topology.rs": {"complexity": 78, "lengths": [54, 45, 28, 24, 11, 4], "wrappers": 0},
    "validator/tests/cases/canonical.rs": {"complexity": 8, "lengths": [34, 19, 10], "wrappers": 0},
    "validator/tests/cases/cli.rs": {"complexity": 10, "lengths": [55, 37, 36, 33, 12, 3], "wrappers": 0},
    "validator/tests/cases/contract.rs": {"complexity": 3, "lengths": [34, 11, 11, 7, 5], "wrappers": 0},
    "validator/tests/cases/expressions.rs": {"complexity": 1, "lengths": [35, 11], "wrappers": 0},
    "validator/tests/cases/importers.rs": {"complexity": 17, "lengths": [57, 50, 43, 40, 39, 27, 25, 21, 9, 8, 3], "wrappers": 0},
    "validator/tests/cases/models.rs": {"complexity": 30, "lengths": [40, 30, 19, 17, 17], "wrappers": 0},
    "validator/tests/cases/pins.rs": {"complexity": 2, "lengths": [19], "wrappers": 0},
    "validator/tests/cases/review_round2.rs": {"complexity": 6, "lengths": [69, 46, 42, 23, 21, 11, 8, 7, 6, 3, 3, 3], "wrappers": 0},
    "validator/tests/cases/runtime.rs": {"complexity": 34, "lengths": [59, 40, 37, 33, 31, 30, 29, 24, 20, 17, 15, 15, 9, 7, 6], "wrappers": 0},
    "validator/tests/cases/suite_e.rs": {"complexity": 57, "lengths": [218, 33, 12, 7, 6, 3], "wrappers": 0},
    "validator/tests/cases/target09.rs": {"complexity": 14, "lengths": [49, 48, 20, 18, 16, 15, 15, 13, 12, 11, 10, 10, 9, 8, 8, 7, 7, 7, 3, 3, 3, 3, 3], "wrappers": 0},
    "validator/tests/integration.rs": {"complexity": 0, "lengths": [], "wrappers": 0},
    "validator/tests/support/pins.rs": {"complexity": 4, "lengths": [25, 5], "wrappers": 0}
  },
  "litter": ["docs/PROGRESS.md"],
  "process_identifiers": {}
}
```
