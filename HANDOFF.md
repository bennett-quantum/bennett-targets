# Handoff: bennett-targets

Round-1 quality update. Milestone 1 (epic **b-targets-4bn**) remains **BLOCKED** on
consumer and contract gaps at the gate pins; the default validator test run is green.

## State
- ISA 0.9 §8 validator: K9 eps_decade, K14 topology (schema, adjacency, default placement, vertex limits, scheme leaves).
  Suite E: 17 executed, 21 published unavailable. E-topology-grid-09 runs here despite its marker, which concerns the reference.
- Four illustrative models, including `superconducting-grid-2d-illustrative`. Azure rates import as count resources, never epsilon.
- Gate: `scripts/pre-push`; repo success is exit **0**, with milestone acceptance
  reported separately. The quality job runs default tests and coverage; the
  orchestrator runs the full pinned consumer gate at merge.
- Test inputs and consumer binaries resolve only through gate.toml's cache.
  Missing cached pins fail; no live sibling fallback exists.
- Exact numeric source ledgers cover every JSON number in all four models,
  including scheme and topology indices. Sidecars cite ISA 0.9. Superconducting
  two-wire timings are exactly 0.0002 seconds.
- Real Qualtran 0.7.0 QFT count output and Microsoft's unmodified estimator report
  have URL/date/hash provenance. Azure imports its explicitly extracted hardware
  parameters; the full report remains a tested unsupported input.
- Five round-2 review regressions reverified; malformed runtime fields now return
  errors, and aggregation has an independent discrete oracle. Measurements and
  test commands: [quality.md](docs/quality.md).

## Open
- **b-targets-dt2**: VM rejects topology targets (`unknown field topology`).
- **b-targets-7dv**: VM refuses target-backed inc8 (`gate-level expansion ... unpublished`), although ref-0.9 is bound and its
  hash recomputed here. Awaits bennett-vm-47t-d.
- **b-targets-38r**: reference refuses all targets; requested surface in b-isa-spec-xg3.
- **b-targets-gyp**: QEC, hierarchy and symbolic import; proposed spec text in b-isa-spec-fd0. Do not implement before a decision.

Evidence: `docs/verification.json`. Orchestrator state: ../bennett/HANDOFF.md.

The existing verification report is historical; this quality pass did not run
the consumer gate or regenerate it. Harness blocker keys `vm-topology`,
`target-backed-inc8` and `reference-target-cli` correspond respectively to
b-targets-dt2, b-targets-7dv and b-targets-38r above. Source diagnostics use these
stable capability names rather than tracking identifiers.
