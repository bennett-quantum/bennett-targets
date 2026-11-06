# Quality targets — round 2 review

**BLOCK — P0-1 (b-targets-7w4).** Reviewed head:
`f14ff57926f6c216e0a9d466e43c4346816611cd`.

Range recorded: `main...job/quality-targets` =
`3ef86a56cb352fbe4bb4f7e7c0b295f23682ee29..f14ff57926f6c216e0a9d466e43c4346816611cd`.
Following the previous review, production review is limited to the fixes
`490edd5..f14ff57` (4a353a1 and f14ff57), plus their affected call paths.
Rechecked the inherited native-model/arity fixes b2a7c25 and 299597e from
`origin/main..main` =
`dd036711936c496f53f6f1a9bebeb686335bf7cc..3ef86a56cb352fbe4bb4f7e7c0b295f23682ee29`.

1. **P0-1 — b-targets-7w4 — resolved model checks still omit the actual angle.**
   At `validator/src/model.rs:236`, the new binding lookup returns `None`
   for a declared `theta` alias even during resolved matching.
   `target.rs:405–409` binds only k/c/eps before the new check at line 437.
   `runtime.rs:76` reduces the supplied angle to a Boolean;
   line 184 does likewise for template leaves, before their parameters are
   evaluated. Consequently model expressions return undecided at
   `model.rs:72` and bypass the zero-epsilon contradiction check at line 292.

   Repro: on the ideal target, use a parametric rx entry with
   `epsilon=c-c`, controls 0..unbounded, and the existing controlled_U
   model envelope. Set `parameter_bindings={"a":"theta"}`; its matrix has
   diagonal real parts −1, off-diagonal real parts
   `{"op":"-","args":[{"var":"a"},{"var":"a"}]}`, and all imaginary
   parts zero. Canonicalize and load, then call
   `expand("rx", &["data"], 0, 0, zero, Some(zero))`.
   It returns an executable leaf with epsilon 0 and params [0].
   ISA §4.4 gives Rx(0)=+I; this model resolves to −I. With c=1 and two
   data operands it also succeeds, although controlled −I differs from
   controlled Rx(0) by operator distance 2. A parametric ry scheme whose
   template is rx(theta) reproduces the same acceptance at theta=0.
   Replacing a−a with literal rational zero makes expansion reject,
   demonstrating the binding-dependent bypass. Full inputs are in the bead.

   Ran independent Rust probes compiled from stdin against this head's freshly
   built library:
   `timeout 20s "$TMPDIR/parametric-probes" --test-threads=4 --nocapture`.
   Exit 101: direct, coherently controlled, and template-leaf assertions
   failed; the literal-matrix rejection control passed. These expressions are
   rational at the supplied exact angle. The accepted K86 snapshot requires
   actual angle/context binding before model checks. This leaves
   b-targets-nf7's root cause incompletely fixed. Carry actual angles through
   both paths while preserving valid parametric models.

**Previous blockers.** b-targets-5pv is fixed: checked addition returns
contextual errors for native/expand, including X, SWAP, CZ and MCX host
boundaries; the allocation-free unit test preserves representable maximum
counts. b-targets-nf7's original k, k−k, table-selected zero and resolved
c cases reject contradictory models; independent scheme-leaf probes also
pass. Its angle-binding case above remains. Reading `git show main:<path>`
confirms the new nonparametric/overflow regressions expose main's old logic;
main also lacks the angle check. No branches switched or worktrees created.

**Verification.** Headless environment; TMPDIR and CARGO_TARGET_DIR under
`.tmp/rev-quality-targets-r2/{tmp,target}`. Ran:

```sh
timeout 170s cargo test --offline --manifest-path validator/Cargo.toml -j3 review_round2:: -- --test-threads=4 --nocapture
timeout 45s cargo test --offline --manifest-path validator/Cargo.toml -j3 --lib -- --test-threads=4 --nocapture
timeout 35s python3 -B -m unittest discover -s scripts -p 'test_quality_ratchet.py' -v
timeout 20s python3 -B scripts/quality_ratchet.py
timeout 20s python3 -B scripts/quality_ratchet.py --measure
timeout 20s "$TMPDIR/contract-probes" --test-threads=4 --nocapture
```

Ten integration regressions, three library tests and nine Python tests pass.
Ratchet CLI passes. The first independent harness exits 101: three probes
pass (original epsilon cases, resolved template binding, overflow); its
angle probe fails. The second harness isolates the three angle failures
above. No existing test was weakened, deleted or ignored; added expectations
are independent. No full gate, gate tool, sibling gate or coverage run.

**Ratchet.** Independently parsed the old/new baselines and compared fresh
measurements: only model.rs, target.rs and review_round2.rs change, exactly
matching measurement. Complexity increases 108→111, 265→274 and 6→14,
respectively, account for binding-aware checks, checked arithmetic and their
tests. The algorithm and wrapper/litter/identifier allowances are unchanged;
no production function exceeds 300 lines. These explicit changes are justified,
with no additional ratchet blocker.

**P2+ nits — b-targets-vdy:** carry forward b-targets-jxz's tracked stale
docs/PROGRESS.md (b-targets-u8u), pre-K86 ISA oracle pin (b-targets-5jl), and
repeated string operator dispatch at model.rs:75–108; no new nits.
Checked every COMMON anti-slop rule; unresolved binding hides the confirmed
P0 above. No further confirmed bug class or scope narrowing.

**Coverage:** closely read both fixes, affected model/epsilon/native/expansion
code, changed tests, previous reviews, ratchet implementation/tests/baseline
and relevant pinned ISA/accepted K86 clauses; skimmed older quality prose.
The ISA pin predates K86; no pinned K86 conformance is claimed.

**Provenance:** implementer gpt-6.1-sol (inherited fixes: Claude Opus 5.5);
reviewer gpt-6.1-sol, fresh session, D9. No implementation edits.
