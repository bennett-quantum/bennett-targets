# Targets gate-repair review, round 1

Reviewed range: `1763730488531556d7e34d7b22da72c830cc53a9..ab4cd24b01db01003aa94cc7cf14cf64a964a904`.
Reviewed head: `ab4cd24b01db01003aa94cc7cf14cf64a964a904`.
Verdict: **CLEAR**. P0/P1: none; no blocking beads.

Both resolvers select the explicit gate layout when supplied, otherwise the cache,
and verify the complete HEAD before returning. Missing paths, missing metadata,
Git failures and mismatches fail loudly; neither path retries a live sibling.
ISA fixture consumers use the Rust resolver; Python ISA and VM reads use the
same Python resolver. Expected commits in the tests come from independent
temporary Git repositories. Assertions, negative coverage and existing tests
remain active; no tests were skipped, removed or weakened.

Main-side comparison (`git show main:scripts/gate_pin.py` and
`git show main:validator/tests/support/pins.rs`) confirms the old code ignored
layouts and accepted mismatched/cache directories without HEAD verification:
the layout-success, mismatch and missing-metadata regressions detect those bugs
(Rust's new explicit-layout argument supplies the test seam).

Verification, with desktop/browser variables unset, job-local TMPDIR and
CARGO_TARGET_DIR:

- `timeout 150s cargo test --offline --manifest-path validator/Cargo.toml -j3 --test integration pin_checks:: -- --test-threads=4`: **5 passed**.
- `timeout 45s python3 -B -m unittest discover -s scripts -p 'test_*.py' -v`: **18 passed**, including seven resolver tests and nine ratchet tests.
- `timeout 20s python3 -B scripts/quality_ratchet.py`: **passed**, 31 source files.
- Bounded inline assertions resolved the actual ISA and VM pins through both cache and a temporary symlinked layout, including an invalid inherited GIT_DIR. Within that layout, `timeout 35s cargo test --offline --manifest-path validator/Cargo.toml -j3 --test integration contract:: -- --test-threads=4`: **5 passed**.
- Parsed TOML comparison proves the sole gate-setting change is `BENNETT_GATE_LAYOUT = "{layout}"`; all seven check commands and both pins are identical. The existing `pin-resolver` check discovers the new tests. The pinned gate expands the layout and applies this environment to each check; no existing check is loosened.
- Baseline comparison confirms only the four changed source/test files receive updated metrics, exactly matching current measurements; all other metrics, wrapper/litter/identifier allowances and the ratchet algorithm are unchanged. HANDOFF.md remains retired with README's ecosystem pointer. COMMON anti-slop rules and plan items 9–11 checked.
- `git diff --check 1763730..ab4cd24`: **passed**. Inspected the main checkout's exact-SHA isolated gate record: all seven checks exited 0; did not rerun the full gate.

P2+ nits: **b-targets-65a** (P3), README.md:55–56 still describes sibling reads as cache-only; mention the explicit gate layout.

Coverage: read closely all six changed files, resolver callers, ratchet implementation/tests, gate configuration and relevant pinned-tool environment/layout code; skimmed unchanged validator cases. The referenced `targets-gatefix.md` brief is absent; read the matching `quality-targets-gatefix.md`.

Provenance: implementer gpt-6.1-sol; reviewer gpt-6.1-sol, fresh session (D9).
