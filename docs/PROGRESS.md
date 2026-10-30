# Milestone 1 acceptance report — 2026-10-30

**BLOCKED, not complete.** Implementation work and all reproducible evidence are
in this repository only. No consumer code was inspected, imported or changed.
No push or Dolt synchronization was performed. Full remit remains in PLAN.

## Deliverables and counts

- PLAN covers validator/canonical identities, vendor models and Qualtran/Azure
  imports, with later publication and integration milestones; pin updated 0.3 → 0.8.
- Independent Rust parser rejects duplicate keys and malformed JSON, emits RFC
  8785 (ECMAScript decimals, UTF-16 key ordering, no LF), hashes canonical bytes
  directly, and validates §8 schema/bindings. Exact rational evaluator handles the
  entire defined AST. Request-time matching, non-nested scheme expansion and
  event-ledger resource/timing/capacity checks are exposed as library APIs.
- **6/6** suite-E target files reproduce canonical bytes and hashes through the
  library and CLI, including both semantically negative canonical targets.
  The independently published hash is checked, not inferred from a consumer.
- **16/16** published E scenarios pass against their structured expected data.
  Scheme correctness-negative remains a T3 counterexample; syntactically valid
  schemes are not thereby certified correct.
- **30/31 VM load comparisons agree**, including the same cited rejection rule;
  **1 divergence** is valid small-exponent JCS. Detailed diagnostics, binary
  hashes and per-model resource outputs are in `docs/verification.json`.
- **0 reference semantic target agreements; 6 explicit staged refusals.** These
  are unavailable surfaces, never passes. Circuits README exposes no target CLI.
- **3/3 models** and **2/2 importer outputs** validate and execute native gates
  in VM on the published registers fixture. Gate count, per-class peak, exact
  makespan and occupancy are asserted for each model.
- The normative §16.3 inc8 object is extracted from the spec and independently
  wrapped/hashes derived by §9.2. It validates and executes untargeted in both
  CLIs. **3/3 target-backed inc8 attempts are blocked**, naming the unpublished
  reference expansion library; registers execution does not replace this criterion.
- All numeric provenance is illustrative, with a sidecar per model: ideal
  all-to-all; superconducting grid class/timing projection (no topology claim);
  all-to-all trapped-ion chain with serial contention. The obsolete bootstrap
  d17 description was removed because it used the wrong schema/string expressions.
- Importers: **7/7 GateCounts fields**; Azure's **10 core fields plus optional
  idleErrorRate** each mapped or explicitly rejected, with published defaults.
  Unknown keys, symbolic/fractional/negative counts, unsupported units, nonzero
  stochastic probabilities and QEC/factory/hierarchy inputs fail with context.
  Large counts are retained exactly using decimal strings. Azure measurement
  time is a named imported quantity, not a claimed runtime latency translation.

## Unresolved beads (all retained in scope)

- **b-targets-dd3**, isa-0.9: §8.2 allows a declared topology but §8.3 publishes
  no topology schema/placement semantics; true 2D-grid description is blocked.
- **b-targets-w8o**, isa-0.9: eps_decade indexing undefined; mirrors ISA f31.
- **b-targets-gyp**, isa-0.9: stochastic/channel error, distinct measurement
  duration, QEC/factory/hierarchical model semantics missing for full imports.
- **b-targets-38r**, isa-0.9: reference target CLI cannot supply required oracle.
- **b-targets-7dv**, isa-0.9: normative inc8 target execution needs the published
  ref expansion library (ISA 8r4 / VM 47t), including hash/ordinals.
- **b-targets-4kf**, consumer-divergence: VM rejects valid small-exponent JCS.

Milestone epic **b-targets-4bn** remains blocked. Plan/importer/evidence tasks are
closed for their delivered mechanics; full validator/model acceptance tasks stay
blocked. No public ISA 0.8 changes or private extensions were introduced.

## Final local gate

Headless `scripts/pre-push`: formatting, strict all-target Clippy, **22 Rust tests
pass / 0 fail / 0 ignored**, build, then CLI verification. **Exit 2** at the final
harness records known acceptance blockers. Missing binaries, novel divergences,
wrong rejection rules or wrong accounting fail immediately (exit 1).
`git diff --check` passes. Exact command:

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-targets/.tmp /home/tobias/Projects/bennett-workspace/bennett-targets/scripts/pre-push
```

## Additional red-green / session surprises

Parameter-presence and predicate-depth tests failed before request matching was
fixed. Huge GateCounts input failed at a host-word ceiling, then passed after
using arbitrary integers and decimal-string report values; incompatible base time
units now reject. CZ is a two-target diagonal gate with zero own controls; its
scheme test failed before that mapping was corrected. Signed repeat bounds are
not forbidden by §8: a negative-i parameter test failed before replacing the
unsigned loop. The illustrative width list [32,64] is not a fixed enum; [16,32,128]
failed before removing that unintended narrowing, then agreed with VM. Explicit
params:null failed a new negative test before optional-array handling was fixed.

`bd init` itself created an automatic bootstrap commit and configured a Dolt
remote; no sync was invoked. Subsequent Beads commits always follow export.
The init-time `bd -C` attempt refused an uninitialized project; init succeeded
with the repository as tool working directory. Implementer progress is kept here
under the sub-agent worklog exception in CLAUDE.md rule 0.

---

# Milestone 1 progress — 2026-10-30

Read brief and repository contracts; read ISA §8/§9/§16, North Star §4–5/§7,
all suite-E fixture data and conformance progress; read consumer CLI documentation.
Reference target CLI is staged; this cannot count as semantic agreement. VM has
native target execution. Circuits has no target CLI documented. Topology and
`eps_decade` require contract clarification, not invented fields.

Red-green evidence and final counts will be appended as work lands.

Schema RED: initial contract tests accepted `{}` and E-epsilon-zero-entry. GREEN:
strict schema and epsilon-zero/mbu negatives pass. Canonical fixture gate: 6/6
exact bytes and hashes, including the independent published 71e1973c… vector.
RFC 8785 UTF-16 ordering, finite binary64 shortest decimals, negative zero and
escaped strings pass. Expression RED: a constant 1/0 nested under `k + ...`
was accepted by schema; GREEN: recursively check constant subexpressions.

Runtime RED: missing expand/aggregate/import_quantities APIs failed the new
runtime test target. GREEN: four runtime groups pass; suite-E parity scheme is
explicitly an accounting witness, not a valid MCX scheme (T3 negative retained).
Importer RED: missing importers module failed compilation; GREEN: both groups
pass, exercising all seven GateCounts fields and ten Azure qubitParams fields,
unknown keys, negative/symbolic/fractional counts, error rates and unsupported
outer Azure fields. Model RED: required target files absent; generated three
canonical descriptions with complete illustrative numeric provenance.

VM native views execution succeeded for ideal/ion. Initial grid times 20 ns /
200 ns rejected valid RFC 8785 small-exponent numbers: b-targets-4kf. A separate
coarse microsecond illustrative grid profile is used for runnable model tests;
regression witness preserves the consumer defect without changing canonical
expectations. A true connectivity-constrained grid remains blocked by b-targets-dd3.
