# ISA 0.8 targets

Three hand-derived canonical descriptions, each with a `.provenance.md` sidecar
covering every numeric field. All hardware values are explicitly illustrative.

- `ideal-all-to-all.json`: exact ideal simulator with unbounded logical wires.
- `superconducting-grid-illustrative.json`: finite data/buffer classes and coarse
  microsecond timings. The intended 4×4 grid connectivity is blocked by the absent
  topology schema (b-targets-dd3); JSON is its class/timing projection only.
- `trapped-ion-illustrative.json`: finite all-to-all chain with serial contention.

Stored bytes are RFC 8785 with no trailing LF. Run the validator and
`scripts/verify.py` headlessly. Every model executes the published registers
fixture's two-toggle launch in the VM, checking native gate count and peak usage.
The original bootstrap d17 file used obsolete string expressions and a gate map;
it was removed rather than passed off as a validated surface-code model.

These descriptions are examples for vendor contributions; actual vendor models
need public numeric sources, capability validation and scheme correctness evidence.
Native lifted/oracle decomposition and true grid placement await published ISA
contracts; they remain acceptance blockers, not supported hardware claims.
