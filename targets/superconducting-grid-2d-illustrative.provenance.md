# superconducting-grid-2d-illustrative: provenance

Every numeric value outside `topology` is **illustrative** and identical to
`superconducting-grid-illustrative.json`; see that sidecar for each value. These
are vendor-neutral models, not claims about a commercial device. No datasheet
accuracy, fidelity, QEC overhead or channel certificate is claimed.
ISA 0.9 §8 defines all field semantics.

| JSON path / values | Provenance and derivation |
| --- | --- |
| topology.vertices 0..15, class data | illustrative row-major 4 × 4 nearest-neighbour grid; count equals classes.data.capacity |
| topology.vertices 16..19, class buffer | illustrative buffer row; count equals classes.buffer.capacity |
| topology.edges, 24 data edges | horizontal (i, i+1) and vertical (i, i+4) neighbours of the grid |
| topology.edges, 4 buffer edges | each bottom-row data vertex 12+c joined to buffer vertex 16+c |

This grid is bipartite: no three vertices are pairwise adjacent, so a native ccx
leaf could never be placed. §8.3 matching takes a `gates` entry before a scheme,
so this model has no ccx entry and carries scheme `ccx-line-7t`: the
textbook Toffoli (7 T, 6 CX) with each c[0]-t CX routed through c[1] by explicit
swaps, so it needs the line c[0]-c[1]-t and its leaves are exact. The author's
correctness obligation (§8.3) is discharged by the 8-column unitary check in
validator/tests/models.rs. Leaf costs follow the gate table; the scheme has no
cost of its own. A control-extended ccx and mcx have no entry here, so §8.3
matching reports a load-time error for them; a vendor model would add schemes.
Routing and movement cost
are deferred to 0.10; explicit `swap` sequences execute and are charged normally.

## Exact numeric source ledger

These are the author-selected illustrative values, in JSON Pointer order.
The source is the construction described above, under ISA 0.9 §8; no hardware
measurement is implied. The validator tests parse this table and compare every
numeric JSON leaf by path and exact rational value, including topology edges
and scheme counts. Equal numbers at different paths remain distinct claims.

| JSON pointer | Exact number | Source and derivation |
| --- | --- | --- |
| `/capabilities/classical_step_time` | `0.00001` | Illustrative controller settings above (ISA 0.9 §8.3) |
| `/capabilities/max_parallel` | `4` | Illustrative controller settings above (ISA 0.9 §8.3) |
| `/capabilities/measurement/feedforward_latency` | `0.001` | Illustrative controller settings above (ISA 0.9 §8.3) |
| `/classes/buffer/capacity` | `4` | Illustrative capacity and occupancy/borrowing weights above (ISA 0.9 §8.3–8.4) |
| `/classes/buffer/dirty_rate` | `0.25` | Illustrative capacity and occupancy/borrowing weights above (ISA 0.9 §8.3–8.4) |
| `/classes/buffer/qubit_second` | `0.25` | Illustrative capacity and occupancy/borrowing weights above (ISA 0.9 §8.3–8.4) |
| `/classes/data/capacity` | `16` | Illustrative capacity and occupancy/borrowing weights above (ISA 0.9 §8.3–8.4) |
| `/classes/data/dirty_rate` | `0.5` | Illustrative capacity and occupancy/borrowing weights above (ISA 0.9 §8.3–8.4) |
| `/classes/data/qubit_second` | `1` | Illustrative capacity and occupancy/borrowing weights above (ISA 0.9 §8.3–8.4) |
| `/gates/0/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/0/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/0/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/0/cost/time` | `0.00002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/0/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/1/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/1/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/1/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/1/cost/time` | `0.00002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/1/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/2/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/2/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/2/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/2/cost/time` | `0.00002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/2/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/3/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/3/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/3/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/3/cost/time` | `0.00002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/3/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/4/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/4/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/4/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/4/cost/time` | `0.00002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/4/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/5/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/5/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/5/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/5/cost/time` | `0.00002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
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
| `/gates/9/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/9/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/9/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/9/cost/time` | `0.00002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/9/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/10/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/10/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/10/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/10/cost/time` | `0.00002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/10/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/11/controls/max` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/11/controls/min` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/11/cost/gate_count` | `1` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/11/cost/time` | `0.00002` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/gates/11/epsilon` | `0` | Illustrative gate counts, timings and exact epsilon above (ISA 0.9 §8.1–8.3) |
| `/ptr_widths_supported/0` | `32` | Illustrative host ABI widths (ISA 0.9 §8.3) |
| `/ptr_widths_supported/1` | `64` | Illustrative host ABI widths (ISA 0.9 §8.3) |
| `/schemes/0/ancillae/count` | `0` | Explicit Toffoli decomposition and wire/control arities (ISA 0.9 §4.4, §8.3) |
| `/schemes/0/realises/controls/max` | `2` | Explicit Toffoli decomposition and wire/control arities (ISA 0.9 §4.4, §8.3) |
| `/schemes/0/realises/controls/min` | `2` | Explicit Toffoli decomposition and wire/control arities (ISA 0.9 §4.4, §8.3) |
| `/topology/edges/0/0` | `0` | Illustrative row-major grid construction above |
| `/topology/edges/0/1` | `1` | Illustrative row-major grid construction above |
| `/topology/edges/1/0` | `0` | Illustrative row-major grid construction above |
| `/topology/edges/1/1` | `4` | Illustrative row-major grid construction above |
| `/topology/edges/2/0` | `1` | Illustrative row-major grid construction above |
| `/topology/edges/2/1` | `2` | Illustrative row-major grid construction above |
| `/topology/edges/3/0` | `1` | Illustrative row-major grid construction above |
| `/topology/edges/3/1` | `5` | Illustrative row-major grid construction above |
| `/topology/edges/4/0` | `2` | Illustrative row-major grid construction above |
| `/topology/edges/4/1` | `3` | Illustrative row-major grid construction above |
| `/topology/edges/5/0` | `2` | Illustrative row-major grid construction above |
| `/topology/edges/5/1` | `6` | Illustrative row-major grid construction above |
| `/topology/edges/6/0` | `3` | Illustrative row-major grid construction above |
| `/topology/edges/6/1` | `7` | Illustrative row-major grid construction above |
| `/topology/edges/7/0` | `4` | Illustrative row-major grid construction above |
| `/topology/edges/7/1` | `5` | Illustrative row-major grid construction above |
| `/topology/edges/8/0` | `4` | Illustrative row-major grid construction above |
| `/topology/edges/8/1` | `8` | Illustrative row-major grid construction above |
| `/topology/edges/9/0` | `5` | Illustrative row-major grid construction above |
| `/topology/edges/9/1` | `6` | Illustrative row-major grid construction above |
| `/topology/edges/10/0` | `5` | Illustrative row-major grid construction above |
| `/topology/edges/10/1` | `9` | Illustrative row-major grid construction above |
| `/topology/edges/11/0` | `6` | Illustrative row-major grid construction above |
| `/topology/edges/11/1` | `7` | Illustrative row-major grid construction above |
| `/topology/edges/12/0` | `6` | Illustrative row-major grid construction above |
| `/topology/edges/12/1` | `10` | Illustrative row-major grid construction above |
| `/topology/edges/13/0` | `7` | Illustrative row-major grid construction above |
| `/topology/edges/13/1` | `11` | Illustrative row-major grid construction above |
| `/topology/edges/14/0` | `8` | Illustrative row-major grid construction above |
| `/topology/edges/14/1` | `9` | Illustrative row-major grid construction above |
| `/topology/edges/15/0` | `8` | Illustrative row-major grid construction above |
| `/topology/edges/15/1` | `12` | Illustrative row-major grid construction above |
| `/topology/edges/16/0` | `9` | Illustrative row-major grid construction above |
| `/topology/edges/16/1` | `10` | Illustrative row-major grid construction above |
| `/topology/edges/17/0` | `9` | Illustrative row-major grid construction above |
| `/topology/edges/17/1` | `13` | Illustrative row-major grid construction above |
| `/topology/edges/18/0` | `10` | Illustrative row-major grid construction above |
| `/topology/edges/18/1` | `11` | Illustrative row-major grid construction above |
| `/topology/edges/19/0` | `10` | Illustrative row-major grid construction above |
| `/topology/edges/19/1` | `14` | Illustrative row-major grid construction above |
| `/topology/edges/20/0` | `11` | Illustrative row-major grid construction above |
| `/topology/edges/20/1` | `15` | Illustrative row-major grid construction above |
| `/topology/edges/21/0` | `12` | Illustrative row-major grid construction above |
| `/topology/edges/21/1` | `13` | Illustrative row-major grid construction above |
| `/topology/edges/22/0` | `12` | Illustrative row-major grid construction above |
| `/topology/edges/22/1` | `16` | Illustrative row-major grid construction above |
| `/topology/edges/23/0` | `13` | Illustrative row-major grid construction above |
| `/topology/edges/23/1` | `14` | Illustrative row-major grid construction above |
| `/topology/edges/24/0` | `13` | Illustrative row-major grid construction above |
| `/topology/edges/24/1` | `17` | Illustrative row-major grid construction above |
| `/topology/edges/25/0` | `14` | Illustrative row-major grid construction above |
| `/topology/edges/25/1` | `15` | Illustrative row-major grid construction above |
| `/topology/edges/26/0` | `14` | Illustrative row-major grid construction above |
| `/topology/edges/26/1` | `18` | Illustrative row-major grid construction above |
| `/topology/edges/27/0` | `15` | Illustrative row-major grid construction above |
| `/topology/edges/27/1` | `19` | Illustrative row-major grid construction above |
