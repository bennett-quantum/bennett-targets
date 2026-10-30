# superconducting-grid-illustrative: provenance

Every numeric value is **illustrative**, hand-derived for deterministic tests;
these are vendor-neutral models, not claims about a commercial device.
No datasheet accuracy, fidelity, QEC overhead or channel certificate is claimed.
ISA 0.8 §8 defines all field semantics.

| JSON path / values | Provenance and derivation |
| --- | --- |
| ptr_widths_supported: 32, 64 | illustrative host ABI widths allowed by ISA |
| classes.data.qubit_second: 1 | illustrative occupancy weight |
| classes.data.dirty_rate: 0.5 | illustrative borrowing weight |
| classes.data.capacity: 16 | illustrative live-qubit limit |
| gates.*.epsilon: 0 | illustrative ideal exact semantics, not hardware infidelity |
| gates.*.cost.gate_count: 1 | illustrative one count for each native leaf |
| gates.*.controls.min/max: 0, 1, 2 or unbounded | own-control arities from ISA §4.4; ideal model permits inherited controls; mcx has variable controls |
| gates one-wire time: 2e-05 seconds | illustrative; x/h/s/sdg/t/tdg/rx/ry/rz |
| gates two-wire time: 0.0002 seconds | illustrative; cx/cz/swap |
| ccx time, where present: 0.0012 seconds | illustrative six times two-wire time, not a decomposition claim |
| max_parallel: 4 | illustrative simultaneous gate limit |
| classical_step_time: 1e-05 seconds | illustrative controller step |
| measurement.feedforward_latency: 0.001 seconds | illustrative classical feedforward delay |

`mbu` is false (reserved in §6.4); `qram` is null (no native QRAM);
measurement is permitted; objective is lexicographic with no numeric weights.
The clocks, capacity and cost weights are chosen independently. There is no
inferred stochastic-error or code-distance model. No cost belongs to a scheme.

Physical intent: a 4 × 4 nearest-neighbour superconducting data grid and a
four-wire buffer. All values are illustrative. classes.buffer.capacity=4,
qubit_second=0.25 and dirty_rate=0.25 are illustrative. Target 0.8 has no
published topology schema (b-targets-dd3), so this JSON carries the class,
capacity and timing projection only. **Grid connectivity is blocked**, not
represented or claimed enforced by the VM. Do not use this projection as a
routing-valid hardware model. Publishing a complete grid target waits for
that contract; no private topology key or fabricated routing cost is used.

This is a deliberately coarse microsecond timing profile, chosen illustratively.
The initial 20 ns / 200 ns profile exposed VM JCS defect b-targets-4kf; its
valid scientific-notation witness remains in scripts/verify.py. No byte or hash
expectation was changed to agree with the consumer. Connectivity still waits
for b-targets-dd3.
