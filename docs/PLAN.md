# Target descriptions implementation plan

The remit is the complete ISA target contract: independent validation and canonical
JCS identities; vendor-contributed, per-hardware descriptions with provenance;
and Qualtran/Azure resource-model ingestion with loss-explicit field mappings.
Only published ISA 0.8 is a cross-repository dependency. No consumer code is read
or copied. The brief supersedes the skeleton 0.3 pin.

## Milestones

1. Canonical parser/hash, exact expressions, all §8 schema/context checks,
   request-time matching, scheme expansion and event accounting; validate/canon
   JSON CLI; full suite-E data checks and CLI differential evidence; three
   illustrative hardware families with provenance; initial concrete Qualtran and
   Azure formats with every field mapped or rejected. Record contract and consumer
   blockers rather than claiming unavailable results. Red-green per feature.
2. Publishable vendor models and importer breadth: verified vendor sources,
   symbolic/hierarchical source-model handling, complete unit conversions and
   channel-error semantics once published. Preserve arbitrary resource vectors.
3. ISA revisions and integration: defined topology, eps_decade, decomposition
   libraries, full scheduler/circuit consumers, scheme correctness certificates,
   imported error models and factories. No North-Star capability is removed.

## Milestone 1 acceptance

Canonical bytes and SHA-256 must match every suite-E target (even schema-negative
fixtures); semantic negatives must reject for their cited sentence. Validation
requires canonical input; canon accepts ordinary JSON and emits an explicit JSON
result containing canonical text and hash. Hash canonical target bytes directly
(§8.1); symbol envelopes (§9.2) do not apply. Decimal costs use exact rationals.
Runtime-dependent expressions are checked in their actual bindings, never with a
made-up probe domain. Schema validation does not assert scheme correctness.

Use consumer CLIs exclusively, fail on unclassified differences, distinguish
reference staged refusals from agreement, and exercise VM native execution and
accounting with generated canonical objects. Circuit placement is conditional on
its CLI exposing target input. A 2D hardware topology is not encoded by inventing
an ISA field: file the absent schema and retain the physical grid in provenance.

## Dependencies

The skeleton is std-only. Correct unrestricted decimal rational arithmetic and
RFC 8785 ECMAScript shortest-number formatting justify num-bigint, num-rational,
num-traits and ryu-js. sha2 supplies audited SHA-256 rather than handwritten crypto.
JSON parsing, duplicate detection, schema, evaluation, importers and harness are
independent implementations here; no ISA or VM code is shared.
