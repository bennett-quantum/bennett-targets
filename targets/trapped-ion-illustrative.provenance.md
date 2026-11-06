# trapped-ion-illustrative: provenance

Every numeric value is **illustrative**, hand-derived for deterministic tests;
these are vendor-neutral models, not claims about a commercial device.
No datasheet accuracy, fidelity, QEC overhead or channel certificate is claimed.
ISA 0.9 §8 defines all field semantics.

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

## Exact numeric source ledger

These are the author-selected illustrative values, in JSON Pointer order.
The source is the construction described above, under ISA 0.9 §8; no hardware
measurement is implied. The validator tests parse this table and compare every
numeric JSON leaf by path and exact rational value, including topology edges
and scheme counts. Equal numbers at different paths remain distinct claims.

| JSON pointer | Exact number | Source and derivation |
| --- | --- | --- |
| `/capabilities/classical_step_time` | `0.000001` | Illustrative controller settings above (ISA 0.9 §8.3) |
| `/capabilities/max_parallel` | `1` | Illustrative controller settings above (ISA 0.9 §8.3) |
| `/capabilities/measurement/feedforward_latency` | `0.0001` | Illustrative controller settings above (ISA 0.9 §8.3) |
| `/classes/data/capacity` | `32` | Illustrative capacity and occupancy/borrowing weights above (ISA 0.9 §8.3–8.4) |
| `/classes/data/dirty_rate` | `0.8` | Illustrative capacity and occupancy/borrowing weights above (ISA 0.9 §8.3–8.4) |
| `/classes/data/qubit_second` | `1` | Illustrative capacity and occupancy/borrowing weights above (ISA 0.9 §8.3–8.4) |
| `/gates/0/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/0/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/0/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/0/cost/time` | `0.00001` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/0/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/1/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/1/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/1/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/1/cost/time` | `0.00001` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/1/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/2/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/2/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/2/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/2/cost/time` | `0.00001` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/2/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/3/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/3/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/3/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/3/cost/time` | `0.00001` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/3/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/4/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/4/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/4/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/4/cost/time` | `0.00001` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/4/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/5/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/5/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/5/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/5/cost/time` | `0.00001` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/5/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/6/controls/max` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/6/controls/min` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/6/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/6/cost/time` | `0.0002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/6/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/7/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/7/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/7/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/7/cost/time` | `0.0002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/7/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/8/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/8/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/8/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/8/cost/time` | `0.0002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/8/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/9/controls/max` | `2` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/9/controls/min` | `2` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/9/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/9/cost/time` | `0.0012` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/9/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/10/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/10/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/10/cost/time` | `0.00001` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/10/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/11/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/11/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/11/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/11/cost/time` | `0.00001` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/11/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/12/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/12/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/12/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/12/cost/time` | `0.00001` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/12/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/13/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/13/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/13/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/13/cost/time` | `0.00001` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/13/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/ptr_widths_supported/0` | `32` | Illustrative host ABI widths (ISA 0.9 §8.3) |
| `/ptr_widths_supported/1` | `64` | Illustrative host ABI widths (ISA 0.9 §8.3) |
