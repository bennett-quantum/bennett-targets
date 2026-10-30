# Milestone 1 progress — 2026-10-30

Read brief and repository contracts; read ISA §8/§9/§16, North Star §4–5/§7,
all suite-E fixture data and conformance progress; read consumer CLI documentation.
Reference target CLI is staged; this cannot count as semantic agreement. VM has
native target execution. Circuits has no target CLI documented. Topology and
`eps_decade` require contract clarification, not invented fields.

Red-green evidence and final counts will be appended as work lands.

Schema RED: initial contract tests accepted `{}` and E-epsilon-zero-entry. GREEN:
strict schema and epsilon-zero/mbu negatives pass. Canonical fixture gate: 6/6
exact bytes and hashes, including the independent published 71e1973c… vector.
RFC 8785 UTF-16 ordering, finite binary64 shortest decimals, negative zero and
escaped strings pass. Expression RED: a constant 1/0 nested under `k + ...`
was accepted by schema; GREEN: recursively check constant subexpressions.

Runtime RED: missing expand/aggregate/import_quantities APIs failed the new
runtime test target. GREEN: four runtime groups pass; suite-E parity scheme is
explicitly an accounting witness, not a valid MCX scheme (T3 negative retained).
Importer RED: missing importers module failed compilation; GREEN: both groups
pass, exercising all seven GateCounts fields and ten Azure qubitParams fields,
unknown keys, negative/symbolic/fractional counts, error rates and unsupported
outer Azure fields. Model RED: required target files absent; generated three
canonical descriptions with complete illustrative numeric provenance.

VM native views execution succeeded for ideal/ion. Initial grid times 20 ns /
200 ns rejected valid RFC 8785 small-exponent numbers: b-targets-4kf. A separate
coarse microsecond illustrative grid profile is used for runnable model tests;
regression witness preserves the consumer defect without changing canonical
expectations. A true connectivity-constrained grid remains blocked by b-targets-dd3.
