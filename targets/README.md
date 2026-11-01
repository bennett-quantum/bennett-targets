# ISA 0.9 targets

Four hand-derived canonical descriptions, each with a `.provenance.md` sidecar
covering every numeric field. All hardware values are explicitly illustrative.

- `ideal-all-to-all.json`: exact ideal simulator with unbounded logical wires.
- `superconducting-grid-2d-illustrative.json`: the 4×4 grid plus buffer row with
  an ISA 0.9 §8.3 `topology`, and a line-routed ccx scheme. The VM refuses it
  (`unknown field topology`, b-targets-dt2), so only the validator checks it.
- `superconducting-grid-illustrative.json`: the same classes and timings with no
  topology, so §8.2 reads it as all-to-all; kept so the VM can execute it.
- `trapped-ion-illustrative.json`: finite all-to-all chain with serial contention.

Stored bytes are RFC 8785 with no trailing LF. Run the validator and
`scripts/verify.py` headlessly. Every model the VM can load executes the published registers
fixture's two-toggle launch in the VM, checking native gate count and peak usage.
The original bootstrap d17 file used obsolete string expressions and a gate map;
it was removed rather than passed off as a validated surface-code model.

These descriptions are examples for vendor contributions; actual vendor models
need public numeric sources, capability validation and scheme correctness evidence.
Target-backed execution of the normative inc8 example awaits the VM (b-targets-7dv).
