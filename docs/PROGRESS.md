# Published suite re-baseline — 2026-10-30

Starting targets HEAD `610f481`, initial published ISA `0b913da` (later
`5663477` during verification), VM starting HEAD
`16caaca` with its JCS fix already present. All cross-repo interaction used
published specs/conformance fixture data and black-box binaries. The verifier
now uses conformance/cases/objects/registers.bisa rather than a fixture under
reference implementation tests. No sibling source was read or copied. No ISA
files were edited, dependencies added, push or Dolt sync performed.

Unknown published suite-E cases now require a published unavailable block with
a non-empty bead-naming reason; the verifier and Rust test print each such id
and reason. Missing blocks fail as `unknown published case: <id>`. All sixteen
previously executable scenarios still run, with the exact 16 pin; upstream
markers cannot hide a known runnable case. Unavailable totals are observed,
not pinned. Actual semantic scenario execution remains in suite_e.rs; the
JSON report labels its directory inventory `suite_e_cases`.

| Observation | Before (recorded) | After (actual run) |
|---|---|---|
| Validator E scenarios (pass / fail / unavailable / total) | 16 / 0 / 0 / 16 | 16 / 0 / 3 / 19 |
| Target canonical bytes and SHA-256 checks | 6 / 6 | 7 / 7 |
| VM load comparisons | 30 agree, 1 divergence / 31 | 31 agree, 0 divergences / 31 |
| Reference semantic target agreements | 0 | 0 |
| Reference staged target refusals | 6 | 7 |
| Native VM model runs / importer runs | 3 / 2 | 3 / 2 |
| Normative target-backed inc8 runs | 3 blocked | 3 blocked |

The same published suite against VM in both engines moves from
445 / 0 / 48 / 493 to **445 / 0 / 55 / 500**. The initial `0b913da`
run was 445/0/54/499; upstream then added E-ref-09-copy-native in `5663477`,
which both consumers classified without code changes. Changed rows are A
21/0/0/21 → 21/0/1/22, C 339/0/0/339 → 339/0/1/340, E 0/0/16/16 →
0/0/19/19, F 33/0/2/35 → 33/0/4/37; B, D, G and H are unchanged.
New published ids: A-measure-draw-order-09, C-multiblock-09,
E-ref-09-qxor-witness, E-topology-grid-09, F-frame-records-09 and
F-ratified-confirmations-09, plus the concurrent E-ref-09-copy-native addition.
These are pending-0.9 witnesses, not promoted
ISA 0.8 executor cases.

## Unavailable validator scenario ids (3 observed; no total pin)

- `E-ref-09-copy-native`: unavailable pending 0.9: **b-isa-spec-8r4**;
  complete library gaps **b-isa-spec-cbk / b-isa-spec-8sc**, draft checkpoint only.
- `E-ref-09-qxor-witness`: unavailable pending 0.9: **b-isa-spec-8r4**;
  the complete catalog is a separate job and ref publication follows acceptance.
- `E-topology-grid-09`: unavailable pending 0.9: **b-isa-spec-wr7**;
  optional topology and physical placements require accepted 0.9.

The topology target's canonical bytes and published target_hash
`960cee71e88da141cc6c2b13cab3f28fce1d50c006e67446aadb592e2bfd0882`
are checked. Validator rejects it with `§8.3 schema: unknown field topology`;
VM rejects it with `§8: unknown field topology`. This is a partial schema
check, neither a validator semantic pass nor a semantic failure. The current
validator admits only the fields enumerated in §8.3, which lacks topology.
The deciding §8.2 sentence says “gates on disjoint wires MAY overlap unless
max_parallel or a topology is declared; all-to-all over logical wires.” It
supplies no topology/placement schema. We do not claim a blanket unknown-field
prohibition for targets from §9.1's object rules. Full topology semantics stay
blocked on the accepted 0.9 contract; no grid/placement support was narrowed.

## Red / green and scratch evidence

RED: original verify.py raises KeyError E-topology-grid-09
(`.tmp/rebaseline-red.log`); original suite_e.rs panics on that case
(`.tmp/rebaseline-scenarios-red.log`); original full gate exits 101 on the
canonical fixture count (7 vs 6; `.tmp/rebaseline-gate-red.log`).
GREEN focused: both suite-E Rust tests pass and direct verify.py exits 2
with only known acceptance blockers (`.tmp/rebaseline-scenarios-green.log`,
`.tmp/rebaseline-verify.log`). A durable Rust regression mutates a scratch copy
and checks empty reason, missing bead and absent unavailable block. Independent
scratch replays of the actual Rust suite and Python verifier fail for each
mutation with the precise expected diagnostic: Rust exit 101, Python exit 1
(`.tmp/rebaseline-negative-checks.json`, corresponding reason logs). All
published data remain unchanged. Scratch selection is available through
BENNETT_TARGETS_E_CASES; temporary copies stay under this repo's .tmp.

## Final gate and ownership blocker

The full, unnarrowed scripts/pre-push currently exits **101**, not the expected
acceptance-only exit 2: validator/tests/canonical.rs still pins six canonical
target fixtures and now observes seven. Formatting and strict all-target Clippy
pass before that failure. The current owned-path extension covers suite_e.rs;
canonical.rs has not been authorized. No gate/test was skipped or weakened to
get past it. Red/final full-gate logs: .tmp/rebaseline-gate-red.log and
.tmp/rebaseline-pre-push.log. The pending minimal fix is a deliberate 6→7
canonical-byte/hash pass re-baseline, checking the topology published hash too.

The verifier was also run independently against both interpreter and JIT VM
binaries; both runs exit **2**, with 31 agreements / zero divergences, all model
and importer checks passing, and exactly the four contract blockers below.
Logs: .tmp/rebaseline-verify.log and .tmp/rebaseline-verify-jit.log. The checked-in
docs/verification.json is emitted by the final actual interpreter run of
scripts/verify.py; it was never hand-edited. It reports the current 19-case
inventory and three unavailable ids. Full Rust suite counts beyond canonical.rs
are not claimed while the full gate stops early; focused suite-E tests pass.

## Remaining acceptance blockers

- **b-targets-dd3 / b-isa-spec-wr7:** topology schema and placement semantics
  are absent from accepted ISA 0.8 (§8.2–§8.3); topology fixture remains unavailable.
- **b-targets-w8o / b-isa-spec-f31:** §8.1 names eps_decade but does not define
  its index in accepted ISA 0.8. Ratified draft witnesses do not change the pin.
- **b-targets-7dv / b-isa-spec-8r4 / bennett-vm-47t:** complete ref expansion
  library and binding are unpublished; all three normative target-backed inc8
  launches refuse lift.const.i8 expansion under §8/§11. The qxor witness is not
  a substitute for the complete catalog.
- **b-targets-38r:** reference CLI refuses all seven target attempts as staged
  deliverable 9, so there are zero reference semantic target agreements.
  The documented circuits CLI likewise exposes no target input.

**b-targets-4kf is closed:** 31/31 load comparisons now agree, including
canonical-small-exponent; the grid native model and imported targets also run.
The separate broader probabilistic/QEC importer contract issue **b-targets-gyp**
remains open; it is not a failure of the current loss-explicit importer runs.
Milestone 4bn and acceptance tasks 4bn.2/4bn.3 remain blocked on their contracts.

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
