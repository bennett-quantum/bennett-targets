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
