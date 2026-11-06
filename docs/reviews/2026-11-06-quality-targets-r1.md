# Quality targets — round 1 review

**BLOCK** — P0-1 and P1-1. Reviewed head:
`a88891a750dbd6016fa218114945bcadd541fb2f`.

Ranges: `main...job/quality-targets` =
`3ef86a56cb352fbe4bb4f7e7c0b295f23682ee29..a88891a750dbd6016fa218114945bcadd541fb2f`
(three commits), plus all nine commits in `origin/main..main` =
`dd036711936c496f53f6f1a9bebeb686335bf7cc..3ef86a56cb352fbe4bb4f7e7c0b295f23682ee29`,
including the previously unchecked Opus round-2 fixes.

1. **P0-1 — b-targets-nf7 — zero-epsilon expressions bypass native-model exactness.**
   `validator/src/model.rs:209` sets `exact` using `constant_zero`, which
   rejects every variable-containing expression as nonconstant; line 283 then
   skips the known rational-matrix contradiction check. Attach the published
   X model with both off-diagonal real numerators changed to `"-1"` to
   `ideal-all-to-all.json`'s X entry. Literal `epsilon=0` rejects correctly;
   `epsilon={"var":"k"}` and `epsilon=k-k` both validate successfully.
   X has k=0, and resolved `native("x", &["data"], 0, 0, zero)` returns a
   charge with epsilon zero for the contradictory −X realisation.
   K86 requires agreement with phase-fixed §4.4 in **every resolved
   zero-epsilon context**, not merely syntactically constant ones. This is
   the evaluated-epsilon rule in the accepted contract snapshot
   `bennett/docs/council/contract-checker-native-model-2026-11-02/round-2/doc.md`
   (“Model requirements depend on evaluated epsilon”), retained in
   `accept-muse-batch2-2026-11-02/round-1/doc.md`.
   Ran an independent Rust probe and a timed Python CLI mutation probe:
   the Rust assertion failed; CLI returned exit 0 / `valid:true` for
   `{"var":"k"}`, hash
   `76c07705830d70a6c8fb39069282d6041460cb86980903d6056ae1eeb09e1319`.
   The same matrix with literal zero returned exit 1.
   Resolve/check the model at actual bindings; do not forbid expression-valued
   epsilon or native-model capability.

2. **P1-1 — b-targets-5pv — arity validation itself can overflow.**
   `validator/src/target.rs:468` computes `c + targets(op)` unchecked.
   On the unmodified ideal target,
   `expand("x", &["data"], 0, usize::MAX, zero, None)` panics with
   “attempt to add with overflow” instead of returning `Err`.
   Only one operand class is allocated. `Target::native` reaches the same
   helper. Ran the independent Rust probe with `catch_unwind`;
   the assertion that the call returns without panicking failed.
   Use checked arithmetic and test the boundary.

The independent probes were compiled from stdin with `rustc --test` against
the freshly built head library, without editing source. Executed:

```sh
timeout 20s "$TMPDIR/contract-probes" --test-threads=4 --nocapture
```

Result: exit 101, both
`a_zero_valued_epsilon_expression_must_reject_negative_x` and
`maximal_controls_must_return_error_without_panicking` failed.
Their complete determining inputs are recorded in the two beads.

**Verification.** Commands ran headless in this worktree, with
`TMPDIR=$PWD/.tmp/rev-quality-targets-r1/tmp` and
`CARGO_TARGET_DIR=$PWD/.tmp/rev-quality-targets-r1/target`:

```sh
timeout 170s cargo test --offline --manifest-path validator/Cargo.toml -j3 -- --test-threads=4
timeout 35s python3 -B -m unittest discover -s scripts -p 'test_*.py' -v
timeout 20s python3 -B scripts/quality_ratchet.py
timeout 25s cargo fmt --manifest-path validator/Cargo.toml -- --check
timeout 100s cargo clippy --offline --manifest-path validator/Cargo.toml -j3 --all-targets -- -D warnings
```

All 65 Rust tests passed (none ignored; fresh compilation 33.98 s, integration
execution 0.64 s); all 14 Python tests, baseline, formatting and Clippy passed.
The ratchet tests actually invoke its CLI on tracked scratch files and assert
exit 1 for longer functions, extra branch mass, wrappers, litter and process
identifiers. The existing five round-2 regression tests pass, but miss the two
boundaries above. Read the new runtime error tests against `git show main:`
source: the old unwraps/slice panic and old vertex-limit diagnostic would fail
them; head passes. No branch switching, full gate, sibling gate or gate tool.

Every target numeric leaf is checked by pointer and exact rational value against
its illustrative source ledger; mutation tests detect swaps, extras and stale
paths. All four sidecars cite 0.9. The checked-in Azure report is byte-identical
to the [pinned Microsoft artifact](https://raw.githubusercontent.com/microsoft/qdk/e5d6dbeabc5b4fcb557269c7aa37a03c79ae4e85/source/resource_estimator/src/system/test_report.json)
(SHA-256 `52ef97bca37dc9739b82683a2c17214dc8aecf5bdab6c74f5c8f6df9d7263f2a`).
Ran `timeout 35s .tmp/quality-targets/external-venv/bin/python -B -` with
`get_cost_value(QFTTextBook(8), QECGatesCost()).asdict()`: Qualtran 0.7.0
independently regenerated T=6, AND=28, Clifford=47, rotation=15,
measurement=28, exactly matching the fixture. Source/date sidecars and Azure's
unchanged hardware projection are present; the full report is explicitly rejected.
Coverage is recorded with its command in `docs/quality.md`; not remeasured here.
HANDOFF.md is deleted and README points to the ecosystem handoff.

**P2+ nits — b-targets-jxz:** retire tracked stale `docs/PROGRESS.md`
(existing b-targets-u8u); select a K86-bearing ISA pin and oracle
(existing b-targets-5jl); parse model expression operators once into a typed
representation (`model.rs:80–113`). All COMMON anti-slop rules and plan items
1–11 checked; no additional confirmed blocking violation or scope narrowing.

**Coverage:** closely read both ranges' production changes, modified tests,
pin/consumer/ratchet scripts, target ledgers and gate configuration; skimmed
historical status prose and the full Azure report's descriptive sections.
The pinned ISA predates K86; model obligations above use the accepted council
contract snapshot, without claiming pinned K86 conformance.

**Provenance:** implementer gpt-6.1-sol (inherited round-2 fixes: Claude Opus 5.5);
reviewer gpt-6.1-sol, fresh session, D9. No production fixes.
