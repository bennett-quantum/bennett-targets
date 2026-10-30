# Handoff: bennett-targets

ISA 0.8 implementation and evidence committed; **milestone 1 remains blocked**.
No push. Sole implementer changed this repository only; consumer code was neither
read nor copied. See docs/PLAN.md, docs/PROGRESS.md and docs/verification.json.

Build/tests/format/Clippy are in scripts/pre-push; it returns 2 at the final
CLI consumer harness because recorded acceptance blockers are not passes.
The reference target CLI is staged (6 refusals, zero semantic comparisons);
VM agrees on 30 load cases but rejects valid small-exponent JCS (b-targets-4kf).
Three models and two importer outputs execute native gates in VM. Normative
inc8 target-backed execution refuses the unpublished ref library (b-targets-7dv).
A real grid needs a topology schema (b-targets-dd3). eps_decade is undefined
(b-targets-w8o); complete stochastic/QEC/hierarchy import needs b-targets-gyp.
Reference CLI surface tracked as b-targets-38r. Epic b-targets-4bn stays blocked.

Next work depends on published contracts / consumer fixes. Re-run the exact
local gate when available; never convert staged refusals or known divergences
to success, omit capabilities, invent a topology field, or change canonical bytes
to match the VM. Full future remit remains in PLAN. Beads must be exported before
committing. No GitHub CI and no Dolt synchronization.
