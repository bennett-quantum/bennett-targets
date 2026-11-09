# Validator quality measurements

Sibling reads use only the commits selected by `gate.toml`. Locally they resolve
through `.tmp/gate/cache/<name>@<sha12>/<name>`; isolated gate checks use
`BENNETT_GATE_LAYOUT/<name>`, supplied by the gate's `{layout}` placeholder.
Both Rust tests and the Python consumer harness verify the selected checkout's
full Git HEAD before reading it. Missing paths, non-repositories and mismatched
HEADs fail; an explicit layout never falls back to cache or live siblings.

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
use stable capability keys, with the open-issue mapping below.

Remaining scope and evidence limits:

- b-targets-38r (`reference-target-cli`), b-targets-7dv (`target-backed-inc8`) and
  b-targets-dt2 (`vm-topology`) remain consumer limitations at the
  existing pins; this pass does not claim a new consumer agreement. The missing
  successful 2D-grid timing baseline was added to the harness, but its conditional
  consumer path needs the merge gate. The other b-targets-5gr nits were addressed;
  its final consumer-dependent verification remains open.
- b-targets-gyp and b-targets-1wr retain the complete QEC/hierarchy/symbolic and
  native-model encoding scope. Microsoft output is explicitly projected to its
  supported hardware parameters; the full report is rejected and preserved.
- The ISA pin predates bennett.native-model/1 (K86). Round-2 native-model tests
  enforce the accepted review contract, while the pinned suite cannot supply a
  K86 oracle. b-targets-5jl asks the orchestrator to repin once the
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
    "scripts/gate_pin.py": {"complexity": 12, "lengths": [27], "wrappers": 0},
    "scripts/quality_ratchet.py": {"complexity": 114, "lengths": [40, 22, 20, 18, 15, 13, 8], "wrappers": 0},
    "scripts/test_consumer_report.py": {"complexity": 1, "lengths": [5, 3], "wrappers": 0},
    "scripts/test_gate_pin.py": {"complexity": 8, "lengths": [17, 15, 8, 7, 6, 5, 4, 4, 4, 3], "wrappers": 0},
    "scripts/test_quality_ratchet.py": {"complexity": 1, "lengths": [13, 11, 11, 8, 8, 8, 7, 7, 7, 6, 5, 5], "wrappers": 0},
    "scripts/verify.py": {"complexity": 103, "lengths": [24, 14, 13, 11, 9, 8, 4, 3, 2, 2, 2], "wrappers": 0},
    "validator/src/expr.rs": {"complexity": 88, "lengths": [81, 49, 23, 16, 12, 9, 8, 8, 7, 6], "wrappers": 0},
    "validator/src/importers.rs": {"complexity": 89, "lengths": [78, 56, 46, 26, 21, 8, 8, 7, 4], "wrappers": 0},
    "validator/src/json.rs": {"complexity": 73, "lengths": [107, 66, 36, 14, 10, 3], "wrappers": 0},
    "validator/src/lib.rs": {"complexity": 2, "lengths": [6, 3], "wrappers": 0},
    "validator/src/main.rs": {"complexity": 28, "lengths": [49, 9], "wrappers": 0},
    "validator/src/model.rs": {"complexity": 119, "lengths": [151, 49, 38, 38, 35, 27, 24, 7, 3], "wrappers": 0},
    "validator/src/query.rs": {"complexity": 15, "lengths": [12, 12, 11], "wrappers": 0},
    "validator/src/runtime.rs": {"complexity": 274, "lengths": [191, 151, 23, 10, 8, 8, 4, 4], "wrappers": 0},
    "validator/src/target.rs": {"complexity": 275, "lengths": [220, 61, 23, 17, 16, 16, 14, 12, 11, 11, 11, 10, 8, 7, 7, 7, 7, 7, 6, 6, 3, 3, 3, 3], "wrappers": 1},
    "validator/src/topology.rs": {"complexity": 78, "lengths": [54, 45, 28, 24, 11, 4], "wrappers": 0},
    "validator/tests/cases/canonical.rs": {"complexity": 8, "lengths": [34, 19, 10], "wrappers": 0},
    "validator/tests/cases/cli.rs": {"complexity": 10, "lengths": [55, 37, 36, 33, 12, 3], "wrappers": 0},
    "validator/tests/cases/contract.rs": {"complexity": 3, "lengths": [34, 11, 11, 7, 5], "wrappers": 0},
    "validator/tests/cases/expressions.rs": {"complexity": 1, "lengths": [35, 11], "wrappers": 0},
    "validator/tests/cases/importers.rs": {"complexity": 17, "lengths": [57, 50, 43, 40, 39, 27, 25, 21, 9, 8, 3], "wrappers": 0},
    "validator/tests/cases/models.rs": {"complexity": 31, "lengths": [40, 30, 19, 17, 17, 16], "wrappers": 0},
    "validator/tests/cases/pins.rs": {"complexity": 11, "lengths": [28, 26, 25, 19, 17, 13, 10, 7, 3], "wrappers": 0},
    "validator/tests/cases/review_round2.rs": {"complexity": 29, "lengths": [69, 46, 42, 40, 29, 27, 26, 26, 23, 22, 22, 22, 21, 18, 17, 14, 13, 13, 12, 11, 10, 9, 8, 7, 6, 3, 3, 3], "wrappers": 0},
    "validator/tests/cases/runtime.rs": {"complexity": 34, "lengths": [59, 40, 37, 33, 31, 30, 29, 24, 20, 17, 15, 15, 9, 7, 7], "wrappers": 0},
    "validator/tests/cases/suite_e.rs": {"complexity": 57, "lengths": [219, 33, 12, 7, 6, 3], "wrappers": 0},
    "validator/tests/cases/target09.rs": {"complexity": 14, "lengths": [49, 48, 20, 18, 16, 15, 15, 13, 12, 11, 10, 10, 9, 8, 8, 7, 7, 7, 3, 3, 3, 3, 3], "wrappers": 0},
    "validator/tests/integration.rs": {"complexity": 0, "lengths": [], "wrappers": 0},
    "validator/tests/support/pins.rs": {"complexity": 14, "lengths": [67, 7], "wrappers": 0}
  },
  "litter": ["docs/PROGRESS.md"],
  "process_identifiers": {}
}
```

## Continuation verification

Verified 2026-11-06T07:04:35Z after reviewing `git diff main...job/quality-targets`
against COMMON's anti-slop rules. The source process-date comment is removed,
and the unchanged Toffoli tolerance now has a named bound and arithmetic rationale.
HANDOFF.md is retired; README points to the ecosystem handoff. The legacy tracked
progress file remains explicitly recorded under b-targets-u8u, outside this job's
owned paths. b-targets-2jb (runtime mutation failures) and b-targets-c3j (numeric
and sample provenance) were filed and closed after local reverification; the
consumer-dependent b-targets-5gr and ISA-pin/oracle b-targets-5jl remain open.

- `/usr/bin/time -f 'default wall=%e seconds' timeout 150s cargo test --offline --manifest-path validator/Cargo.toml -j3 -- --test-threads=4`: **65 passed**, none ignored, **2.91 seconds** including compilation.
- `timeout 30s cargo fmt --manifest-path validator/Cargo.toml -- --check`: passed.
- `timeout 120s cargo clippy --offline --manifest-path validator/Cargo.toml -j3 --all-targets -- -D warnings`: passed.
- `timeout 30s python3 -m unittest discover -s scripts -p 'test_*.py' -v`: **14 passed**, including nine metric/ratchet tests.
- `python3 scripts/quality_ratchet.py`: passed for **31 source files**, with one pre-existing public forwarding wrapper, one legacy litter path and zero product process identifiers.
- The exact quality gate argv in a TMPDIR scratch repository passed the baseline
  (exit 0), then rejected lengths `[5] -> [7]` and complexity `1 -> 3` (exit 1).
  Evidence: `.tmp/quality-targets/ratchet-red-green.log`; no worsened scratch file
  was placed in the tree. The HANDOFF retirement/pointer check also ran red then green.

These commands use the headless job environment above. LLVM coverage remains the
93.55% measurement recorded for the first attempt; this continuation adds the
Python gate and documents the existing floating-point test bound. The full
pinned consumer/push gate and independent/council review remain the orchestrator's
merge checks; no new consumer agreement is claimed here.


## Evaluated-epsilon model checks

Verified 2026-11-06T07:21:04Z against pinned ISA ecaaac060c44e317d1181345fa6b8e38c096c099
§§4.4/8.1/8.3 and the accepted K86 evaluated-epsilon rule in
`bennett/docs/council/contract-checker-native-model-2026-11-02/round-2/doc.md`.
The pin predates K86; b-targets-5jl still owns selecting its published oracle.
The earlier `origin/main..main` native-model fix shares this root cause.

Three new tests failed before the fix: loading a contradictory -X with epsilon
`k`, matching a contradictory model at achieved zero, and binding `eps` in a
resolved exact model. Loading now uses opcode-fixed `k` and singleton-control
bindings when available; native matching checks the actual evaluated epsilon
and resolves rational model parameter aliases before returning a charge.
Unresolved analytical model expressions remain a checker obligation. Correct X
models with expression-valued epsilon and legal positive-epsilon -X contexts
remain admitted. Additional tests cover `k-k`, `c-c`, table-selected zero,
controlled X-family forms and post-load document mutations.

`timeout 60s cargo test --offline --manifest-path validator/Cargo.toml -j3 --test integration review_round2:: -- --test-threads=4`
passed all nine tests (the existing five plus four new model regressions).
`timeout 90s cargo clippy --offline --manifest-path validator/Cargo.toml -j3 --all-targets -- -D warnings`
and formatting passed. Before the fourth test, the complete Rust run passed
68 tests (2 unit and 66 integration).

The explicit baseline update above is limited to model.rs, target.rs and
review_round2.rs, measured with `python3 -B scripts/quality_ratchet.py --measure`.
It accounts for context-aware model checks, the binding-aware epsilon predicate,
and their regression tests. The ratchet algorithm, litter allowance and
identifier allowance are unchanged; these baseline changes need the same
independent review as the correctness fix. No function exceeds 300 lines.


The arity follow-up replaces the shared `c + targets(op)` computation from
`origin/main..main` commit 299597e with checked addition. Overflow now returns a
host-limit error naming the operation and total controls. New regressions failed
with an addition panic before this fix, then passed for both `native` and
`expand`, including two-target overflow at `usize::MAX - 1`. The allocation-free
helper test admits representable counts up to `usize::MAX` and distinguishes
ordinary arity mismatch from overflow. Only target.rs and review_round2.rs
receive the corresponding explicit baseline changes for this fix and its tests.


## Round-2 final verification

Measured 2026-11-06T07:24:34Z in the same headless pinned environment. Both blocking
review beads b-targets-nf7 and b-targets-5pv are closed with the commands and
results below; b-targets-jxz remains open (its fixes exceed one line), along
with the existing ISA-oracle b-targets-5jl and stale-progress b-targets-u8u.
No gate pin or ratchet algorithm changed. Only the three modified Rust files
received explicit structural baseline updates for independent review.

```sh
/usr/bin/time -f 'default wall=%e seconds' timeout 150s cargo test --offline --manifest-path validator/Cargo.toml -j3 -- --test-threads=4
timeout 35s python3 -B -m unittest discover -s scripts -p 'test_*.py' -v
timeout 20s python3 -B scripts/quality_ratchet.py
timeout 25s cargo fmt --manifest-path validator/Cargo.toml -- --check
timeout 100s cargo clippy --offline --manifest-path validator/Cargo.toml -j3 --all-targets -- -D warnings
timeout 180s cargo llvm-cov --offline --manifest-path validator/Cargo.toml -j3 --json --output-path .tmp/quality-targets/coverage-round2.json -- --test-threads=4
```

All passed. Default Rust tests: **71 passed** (3 unit, 68 integration), none
ignored, **0.40 seconds** including the warm build. Python: **14 passed**.
LLVM executable-line coverage: **93.86% (1834/1954 lines)**, including in-file
unit tests; all 71 Rust tests also passed under instrumentation. The updated
ratchet covers 31 source files, 212 functions and complexity mass 1460; one
existing public forwarding wrapper, one existing litter path and zero product
process identifiers remain. The largest production functions are schema (220
lines), aggregate (191), expand (152), and realisation_model (144). The full
consumer/push gate and independent review remain the orchestrator's merge work.

## Actual-angle model resolution

Verified 2026-11-06T13:24:52Z against the same pinned ISA §§4.4/8.1/8.3 and
accepted K86 text cited above. This repairs b-targets-7w4. The ISA pin still
predates K86; no new pinned K86 oracle agreement is claimed.

With production at reviewed head 4376395, the direct, coherently controlled,
and template-leaf regressions all failed because a model resolving to −I was
accepted for Rx(0). The same tests now reject for all tested axes. The two
positive rotation tests passed before the fix and remain green. Separate
red-first tests exposed missing declared context bindings and empty template
parameter lists being accepted.

Native matching now carries the exact angle value into the model environment.
Template expressions resolve before leaf matching in the parent theta/repeat-i
scope; the resulting leaf angle enters the leaf's own k/c/eps context.
Declared template parameters require exactly one angle. A resolved model check
errors on any missing declared k/c/eps/theta binding, even an unused alias;
schema-time unknowns remain allowed. The §4.4 ideal at zero is +I for each of
Rx, Ry and Rz, so correct rational identity models remain admitted.

Positive nonzero models are hand-derived from P²=I:
exp(−i theta P/2) = cos(theta/2) I − i sin(theta/2) P.
Direct and controlled tests use theta=3/4 radians; the Ry scheme executes
S†, Rx(theta/3), Rx(2 theta/3), S, with repeated leaf angles exactly 1/4
and 1/2. An additional Rz(theta−theta) leaf at zero inside that nonzero
scheme distinguishes the leaf binding from its parent's angle. Analytical
trigonometric expressions still require independent checker reconstruction;
these tests establish valid model admission and exact parameter propagation,
not an interval accuracy proof.

Final commands, in the headless environment documented above:

```sh
/usr/bin/time -f 'default wall=%e seconds' timeout 150s cargo test --offline --manifest-path validator/Cargo.toml -j3 -- --test-threads=4
timeout 35s python3 -B -m unittest discover -s scripts -p 'test_*.py' -v
timeout 20s python3 -B scripts/quality_ratchet.py
timeout 25s cargo fmt --manifest-path validator/Cargo.toml -- --check
timeout 100s cargo clippy --offline --manifest-path validator/Cargo.toml -j3 --all-targets -- -D warnings
timeout 180s cargo llvm-cov --offline --manifest-path validator/Cargo.toml -j3 --json --output-path .tmp/quality-targets/coverage-round3.json -- --test-threads=4
```

All passed: **79 Rust tests** (4 unit, 75 integration), none ignored,
**3.38 seconds** including the warm build; **14 Python tests**; LLVM
executable-line coverage **94.23% (1879/1994 lines)**. Formatting, strict
Clippy and git diff --check passed.

The explicit structural baseline update changes only model.rs, runtime.rs,
target.rs and review_round2.rs. Their complexity masses are respectively
119, 274, 275 and 29; total mass is 1488 across 224 functions and 31 files.
The increases account for angle binding, missing-binding/parameter errors,
zero-angle ideals and regressions. No function exceeds 300 lines. Ratchet
algorithm, one existing wrapper, legacy litter allowance and zero identifier
allowance are unchanged. Nonblocking b-targets-jxz/b-targets-vdy remain open,
including the existing b-targets-5jl pin and b-targets-u8u stale-document
items. Full consumer/push gating and independent review remain merge work.

## Pinned sibling run-layout resolution

The isolated gate creates a clean consumer worktree with pinned siblings beside
it; it does not create a cache inside that worktree. The previous Rust and Python
resolvers only understood the cache layout. `gate.toml` now supplies
`BENNETT_GATE_LAYOUT = "{layout}"`, and both resolvers honour that selection.
The Python consumer harness uses the same resolver for its ISA and VM pins;
all Rust ISA fixture reads use the Rust resolver. Both check full Git HEAD,
reject missing Git metadata, and ignore inherited `GIT_*` overrides when
querying the selected checkout. Errors name the expected commit, selected path
and unused live sibling path, plus the actual HEAD on a mismatch.

Red-first checks used real temporary Git repositories. Before the behavior
change, the five focused Rust tests had four failures; the seven Python tests
reported three failures and two errors. They now cover successful cache and
explicit-layout resolution, mismatched HEADs in both layouts, absent metadata,
and refusal to use a matching cache/live sibling when a selected layout is
missing. The Rust resolver gained an explicit layout argument as a test seam
before the red run; its old cache-only behavior was retained for that run.

Local verification in the headless environment above:

```sh
timeout 150s cargo test --offline --manifest-path validator/Cargo.toml -j3 -- --test-threads=4
timeout 30s cargo fmt --manifest-path validator/Cargo.toml -- --check
timeout 100s cargo clippy --offline --manifest-path validator/Cargo.toml -j3 --all-targets -- -D warnings
timeout 35s python3 -B -m unittest discover -s scripts -p 'test_*.py' -v
timeout 20s python3 -B scripts/quality_ratchet.py
```

All passed: **83 Rust tests** (4 unit, 79 integration), none ignored, and
**18 Python tests**. The baseline update is confined to gate_pin.py,
test_gate_pin.py and the two Rust pin support/test files: 31 files,
239 functions, complexity mass 1518. Wrapper, litter and identifier
allowances and the ratchet algorithm are unchanged.

The pinned gate tool supports `run --repo <main-checkout> --sha <commit>` for
a committed branch SHA. That runs all seven checks under isolation without
pushing; its per-check results belong in the main checkout's
`.tmp/gate/records/<commit>.json` and this job's ignored progress file.


## Ideal circuit target publication

The renamed `ideal-quantum-circuit.json` retains the previous canonical JSON
bytes and SHA-256 `d602ef018d413ffcdea8f0334acbb0b6dd373ba09a325665afb181fa9f6e0cbc`.
Red-first integration checks failed for the missing published filename and
required description before the rename. The updated model/provenance check and
a new description check cover both READMEs and the ideal provenance sidecar.
The published topology case still asserts that absent topology permits the
non-adjacent placement; its descriptive fixture label is diagnostic context.

`cargo test --offline --manifest-path validator/Cargo.toml -j2 --test integration -- --test-threads=2`
passed **80 integration tests**, including all seventeen executable published
E scenarios. Formatting, integration-target strict Clippy and the quality
ratchet passed. Only the owned `models.rs`, `runtime.rs` and `suite_e.rs` baseline
rows above changed: one new description test and formatting for the longer
filename/placement assertion. Other baseline rows and allowances are unchanged.

This is integration evidence, not a full-gate receipt. The embedded unit test
in `validator/src/model.rs` and the consumer harness in `scripts/verify.py`
still name the removed artifact; both are outside this publication child's
owned paths and are tracked by `bennett-targets-yb5`. They must migrate before
the default Rust suite and consumer gate can pass. `docs/verification.json`
remains historical until the corrected harness is run.
