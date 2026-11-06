# Quality targets — round 3 review

**CLEAR — no P0/P1 findings.** Reviewed head:
`8a56cd95e5f9f13d17b999b405dbee4b85fb0034`.

Recorded range `main...job/quality-targets`:
`3ef86a56cb352fbe4bb4f7e7c0b295f23682ee29..8a56cd95e5f9f13d17b999b405dbee4b85fb0034`.
This final review checks the fix
`437639552850bf1464658f7add11b750f4b4dee1..8a56cd95e5f9f13d17b999b405dbee4b85fb0034`
and its call paths, and reproduces all earlier P0/P1 cases.

**Previous blockers.** b-targets-7w4 is fixed: independent probes using its
`a -> theta`, `a-a` off-diagonal expressions and `epsilon=c-c` now reject
−I for Rx(0) directly, under one coherent control, and in a ry scheme's
rx leaf. Native matching receives the exact angle at target.rs:396–415;
runtime.rs:184–195 evaluates each template angle before matching; model.rs:238–244
requires every declared binding in resolved contexts. The zero-angle ideal
is +I, as §4.4 requires. Missing declared k/c/eps/theta bindings and empty or
multiple template angles have rejecting regressions.

The original b-targets-nf7 k, k−k and table-selected zero cases reject −X;
resolved c cases still reject in the focused regressions. b-targets-5pv
remains fixed: independent native/expand calls for X, SWAP, CZ and MCX at
overflowing host control counts return errors without panicking, while the
allocation-free unit test preserves the representable maximum.

Valid +I models at zero remain admitted on direct, controlled and template
paths. Hand-derived analytical rotations at theta=3/4 preserve that exact
angle; the checked-in repeated scheme preserves leaf angles 1/4 and 1/2.
The independent analytical scheme uses two 3/8 leaves. The additional
zero-angle Rz leaf inside a nonzero parent distinguishes leaf from parent
binding. These positive checks establish admission and parameter propagation;
transcendental model reconstruction remains a checker obligation.

**Verification.** Headless environment; TMPDIR and CARGO_TARGET_DIR are
`$PWD/.tmp/rev-quality-targets-r3/{tmp,target}`. Ran:

```sh
timeout 170s cargo test --offline --manifest-path validator/Cargo.toml -j3 review_round2:: -- --test-threads=4 --nocapture
timeout 45s cargo test --offline --manifest-path validator/Cargo.toml -j3 --lib -- --test-threads=4 --nocapture
timeout 20s "$TMPDIR/angle-probes" --test-threads=4 --nocapture
timeout 35s python3 -B -m unittest discover -s scripts -p test_quality_ratchet.py -v
timeout 20s python3 -B scripts/quality_ratchet.py
timeout 25s cargo fmt --manifest-path validator/Cargo.toml -- --check
git diff --check 4376395..8a56cd9
```

All passed: 17 focused integration tests, four library tests, seven independent
Rust probes compiled from stdin against the freshly built head library, and
nine Python tests. Reading `git show main:<path>` and the pre-fix code confirms
the negative angle regressions would fail there: theta was reduced to a
Boolean and model variables stayed undecided. No branch switching, worktree
creation, full gate, gate tool, sibling gate or coverage rerun.

Added tests use independent §4.4 identities and exact rational expectations;
none was weakened, removed or ignored. Independently parsed old/new baselines
and compared fresh measurement: only model.rs, runtime.rs, target.rs and
review_round2.rs change, with complexity 111→119, 270→274, 274→275 and 14→29.
The changes match the binding/error checks and regressions. Ratchet algorithm,
wrapper/litter/identifier allowances remain unchanged; no production function
exceeds 300 lines. Checked every COMMON anti-slop rule; no additional confirmed
bug class or scope narrowing.

**P2+ nits — b-targets-116:** carry forward stale tracked docs/PROGRESS.md (b-targets-u8u), pre-K86 ISA pin/oracle (b-targets-5jl), and repeated string operator dispatch at model.rs:75–108; no new nits.

**Coverage:** closely read the entire fix diff, model/native/expansion call
paths, all added tests, earlier reviews, ratchet implementation/baselines,
pinned ISA §§4.4/8.1/8.3 and accepted K86 clauses; skimmed earlier audit/status
prose. The ISA pin predates K86; no pinned K86 conformance is claimed.

**Provenance:** implementer gpt-6.1-sol; reviewer gpt-6.1-sol, fresh session,
D9. Review only; no production fixes.
