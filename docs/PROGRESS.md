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
