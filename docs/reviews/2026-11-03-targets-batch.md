# Targets batch review — 2026-11-03

**BLOCK** — two P0 and three P1 findings. Reviewed all 22 commits in
`41a3f18f971d58eb1cead43f3b8c0b8d859053ea..3a5abda184dc9d9f3da993482292b6880fdf62f3`;
reviewed HEAD is `3a5abda184dc9d9f3da993482292b6880fdf62f3`. Contract: published ISA 0.9, especially §8.

All findings were reproduced with the following independent contract probes
(exit 101: six failed assertions, each demonstrating the stated defect):

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER \
  TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-targets/.tmp/rev-targets/tmp \
  CARGO_TARGET_DIR=/home/tobias/Projects/bennett-workspace/bennett-targets/.tmp/rev-targets/target \
  timeout 120s cargo test --offline \
  --manifest-path /home/tobias/Projects/bennett-workspace/bennett-targets/.tmp/rev-targets/repro/Cargo.toml \
  -j4 --test review -- --test-threads=4 --nocapture
```

1. **P0 — b-targets-c5w — imported rates silently change.**
   `validator/src/importers.rs:14` canonicalizes rate values copied at lines 269–276 without checking preservation.
   Azure `oneQubitGateErrorRate=0.10000000000000001` imports successfully, but the consumed X-leaf rate is `0.1`.
   Probe: `azure_numeric_rates_must_not_change_on_serialization`.
   A separate timed CLI import of `.tmp/rev-targets/azure-underflow.json` also returned exit 0 and changed positive `1e-400` to zero.

2. **P0 — b-targets-9oc — placement loses the resolved class.**
   `validator/src/topology.rs:91` checks adjacency without checking operand classes.
   With vertex 0 slow, vertex 1 fast, and X times fast=1/slow=100,
   expanding a fast X and placing it on vertex 0 succeeds; aggregation also succeeds with time 1.
   ISA §8.3/§11 require rejection of that class mismatch.
   Probe: `explicit_placement_must_match_resolved_class`.

3. **P1 — b-targets-z7b — published native models are refused.**
   `validator/src/target.rs:187` admits only `params` as an optional gate field.
   The published exact X `bennett.native-model/1` model, with resolved `operand_map=[0]`,
   returns `unknown field realisation_model`; §8.3 K86 explicitly requires upgraded closed-schema readers.
   Probe: `published_native_model_must_load`.

4. **P1 — b-targets-9vu — malformed decimals become valid costs.**
   `validator/src/expr.rs:18` derives scale from the first decimal point, then removes every point.
   Azure duration `"1.2.3 us"` imports successfully as `1.23e-7` seconds instead of rejecting malformed numeric text.
   Probe: `malformed_decimal_duration_must_reject`.

5. **P1 — b-targets-4z8 — native operand arity is unchecked.**
   `validator/src/runtime.rs:151` accepts a scheme template `cx` with only `wires=["t"]`,
   returning an executable CX leaf with one wire. The direct native branch at line 61 also returns an X leaf with no target for `expand("x", [], 0, 0, 0, None)`.
   These violate §4.4 structural arities before scheme correctness is considered.
   Probes: `template_native_operand_arity_must_reject`, `native_operand_arity_must_reject`.

**Verification:** 41 existing focused tests passed: expressions/runtime/models/importers/target09 (33),
canonical/contract (7), and `suite_e::all_seventeen_published_e_scenarios` (1).
Each Cargo run used `timeout`, `-j4`, and `--test-threads=4`; exact commands are in `.tmp/rev-targets/PROGRESS.md`.
No full gate or sibling engine run; no production fixes.

**Nits:** b-targets-5gr collects the missing future grid timing baseline, tests bypassing the claimed operation,
stale version/exit-code prose, source bead references, a redundant hash assertion, and illustrative timing decimal drift.

**Coverage:** closely read every Rust source/test, all four target documents and numeric sidecars, importer fixtures,
gate/test scripts and manifests; skimmed historical/status prose. Checked the North Star and mixed fat-object constraint;
this repo adds no host/device object-format decision.

**Provenance/R14:** reviewer gpt-6.1-sol xhigh under R14's Codex quota direction; the orchestrator queues a Claude cross-check of Codex-authored work.
Claude trailers: `fa890b618`, `1b18d05fb`, `fd74ffc36`, `a0a95dd5c`, `0acda4485`, `610f48166`, `4e5ad05c1`, `5113d9aca`, `4a74c6275`, `0e9cbb33c`, `c788cbae4`, `dd7e657d7`, `dd0367119`, `3a5abda18`.
No commit has an `Agent:`/`codex job` marker; unmarked provenance remains unconfirmed: `22f79a7e0`, `de74108a5`, `6c610e895`, `99d8afb93`, `4bcbbb5a8`, `169b1b12d`, `7c0465d70`, `69eb453cd`.

## Round 2 (fixer)

Fixer: Claude Opus 5.5 (code-quality policy rule 3; round-2 check by the opposite model). All five
regressions live in `validator/tests/review_round2.rs`; each was run red against the reviewed
code before its fix and is green after.

- **b-targets-c5w (P0).** The importer checks every imported number against the rational its
  RFC 8785 text denotes, per Azure field and again over the whole produced target; a value that
  would round (`0.10000000000000001`, positive `1e-400`) fails loudly, never rounds.
  Test `imported_rates_are_preserved_exactly_or_rejected`.
- **b-targets-9oc (P0).** Leaves record each wire's resolved operand class. Placement rejects a
  wire on a vertex of another class, and two wires of one expansion on the same vertex.
  Test `placement_must_match_the_resolved_operand_class`.
- **b-targets-z7b (P1).** Gates entries admit `realisation_model`. The reader checks
  `bennett.native-model/1` against the accepted field set (version, kind, arity, basis,
  parameter_bindings, control_models, data). `b-isa-spec-wk0k` is still open, so `operand_map`
  is tolerated only when it is a permutation that resolves an admitted c. At constant zero
  epsilon, a Gaussian-rational matrix must equal the exact §4.4 base (−X rejects). Unknown
  versions are reported as unsupported. Encodings that 0.9 does not publish reject and cite
  `b-targets-1wr`. Test `published_native_realisation_model_loads_and_is_verified`.
- **b-targets-9vu (P1).** Decimal text must follow the JSON number grammar.
  Test `malformed_decimal_text_rejects`.
- **b-targets-4z8 (P1).** Every native match (the direct path, template leaves and
  `Target::native`) enforces §4.4 own controls and c + targets operands. The existing
  `runtime.rs` division-by-zero probe passed a one-operand cx; it now passes two, and its
  assertion is unchanged. Test `native_operand_arity_is_checked`.
