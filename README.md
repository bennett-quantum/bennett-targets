# bennett-targets

Target descriptions (bennett-isa-spec section 8) per hardware model,
**contributed by vendors**, plus importers for existing resource-model formats
(Qualtran, Azure Quantum Resource Estimator).

- `targets/` canonical-JSON (RFC 8785) target descriptions.
- `validator/` Rust crate `bennett-targets-validator`: `validate(json) -> Result<(), String>`.
  Currently a stub (parse check only); it will grow to full ISA-spec section 8 and
  conformance suite E checks (JCS form, cost-expression language, legality coverage).
- `importers/` (future) Qualtran / Azure model importers emitting the `imports` map.

Depends on: `bennett-isa-spec` only.
