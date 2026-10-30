# trapped-ion-illustrative: provenance

Every numeric value is **illustrative**, hand-derived for deterministic tests;
these are vendor-neutral models, not claims about a commercial device.
No datasheet accuracy, fidelity, QEC overhead or channel certificate is claimed.
ISA 0.8 §8 defines all field semantics.

| JSON path / values | Provenance and derivation |
| --- | --- |
| ptr_widths_supported: 32, 64 | illustrative host ABI widths allowed by ISA |
| classes.data.qubit_second: 1 | illustrative occupancy weight |
| classes.data.dirty_rate: 0.8 | illustrative borrowing weight |
| classes.data.capacity: 32 | illustrative live-qubit limit |
| gates.*.epsilon: 0 | illustrative ideal exact semantics, not hardware infidelity |
| gates.*.cost.gate_count: 1 | illustrative one count for each native leaf |
| gates.*.controls.min/max: 0, 1, 2 or unbounded | own-control arities from ISA §4.4; ideal model permits inherited controls; mcx has variable controls |
| gates one-wire time: 1e-05 seconds | illustrative; x/h/s/sdg/t/tdg/rx/ry/rz |
| gates two-wire time: 0.0002 seconds | illustrative; cx/cz/swap |
| ccx time, where present: 0.0012 seconds | illustrative six times two-wire time, not a decomposition claim |
| max_parallel: 1 | illustrative simultaneous gate limit |
| classical_step_time: 1e-06 seconds | illustrative controller step |
| measurement.feedforward_latency: 0.0001 seconds | illustrative classical feedforward delay |

`mbu` is false (reserved in §6.4); `qram` is null (no native QRAM);
measurement is permitted; objective is lexicographic with no numeric weights.
The clocks, capacity and cost weights are chosen independently. There is no
inferred stochastic-error or code-distance model. No cost belongs to a scheme.

Physical intent: illustrative all-to-all ion chain with a shared entangling
operation controller (max_parallel=1). This models serial contention only.
