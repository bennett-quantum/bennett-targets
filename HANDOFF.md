# Handoff: bennett-targets

Updated 2026-11-01. Milestone 1 (epic **b-targets-4bn**) remains **BLOCKED** (checked with `bd show`).

## State
- ISA 0.8 target validator, three illustrative canonical target models and two importers (Qualtran, Azure) are
  committed. Recorded evidence: `docs/verification.json`, `docs/PROGRESS.md`; plan: `docs/PLAN.md`.
- Gate: `scripts/pre-push` (fmt, clippy, cargo test, build, `scripts/verify.py`). It exits **2 by design** at the
  consumer harness while acceptance blockers are open: a recorded blocker is not a pass. The orchestrator has
  proposed exit 0 with a loud report; not decided. This session did not rerun the gate.
- `docs/verification.json` has an uncommitted modification in the working tree that this handoff does not own.

## Open blockers (all OPEN as of 2026-11-01)
- **b-targets-dd3**: no topology schema in ISA 0.8, so no real connectivity-constrained grid target.
- **b-targets-w8o**: `eps_decade` indexing undefined.
- **b-targets-7dv**: target-backed normative inc8 example needs the published ref library expansion.
- **b-targets-38r**: reference CLI target evaluation unavailable (as recorded in verification.json, refuses targets).
- **b-targets-gyp**: import of probabilistic hardware errors / QEC models undefined.
- Closed: b-targets-4kf (VM RFC 8785 small-exponent numbers; VM comparisons agreed after its fix).

## Next
Contract fixes for the blockers belong to the ISA spec, not here; never invent fields. When they land, rebaseline
fixtures, rerun `scripts/pre-push`, and update verification.json. Orchestrator state: ../bennett/HANDOFF.md.
