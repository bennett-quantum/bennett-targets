# bennett-targets

Independent Bennett ISA 0.8 target validator, canonical descriptions and first
Qualtran/Azure adapters. See [PLAN](docs/PLAN.md), [progress](docs/PROGRESS.md),
[models](targets/README.md), and [field mappings](importers/README.md).

```sh
/home/tobias/Projects/bennett-workspace/bennett-targets/scripts/pre-push
```

The local gate unsets desktop/browser variables and uses this repo's `.tmp/`.
Build via Cargo with the absolute `validator/Cargo.toml` manifest. The binary is
`validator/target/debug/bennett-targets-validator`:

- `validate <file>`: canonical input and schema/context checks, JSON valid/hash.
- `canon <file>`: strict JSON to canonical bytes/hash encoded in a JSON result;
  canonicalization does not assert semantic validity (negative fixtures work).
- `import-qualtran <base-target> <input-json>` / `import-azure ...`: canonical
  target plus loss-explicit field mapping and converted quantities.

Errors are JSON on stderr, empty stdout, exit 1. Canonical text inside a result
is a string: extract it without appending LF. The hash is SHA-256 of exactly the
canonical target bytes (§8.1), not the §9 symbol envelope or CLI output line.

The library provides `validate`, `canon`, exact `expr` evaluation, `Target::load`,
native matching, template expansion, import conversion and scheduled leaf-event
aggregation. Context-dependent errors are checked with actual request bindings;
schema validation does not prove all possible requests valid or prove a scheme's
unitary correct. Field/context diagnostics cite the relevant ISA sentence.

Acceptance is **blocked**, not complete: ISA 0.8 has no topology schema and
undefined eps_decade indexing; the reference CLI refuses all targets; consumer
schema/JCS differences are retained in [verification](docs/verification.json).
The consumer harness returns exit 2 for those known acceptance blockers and
fails immediately on missing binaries, broken witnesses or unexpected run results.
Circuits' documented CLI does not yet accept targets. No other repo is modified.

Small dependencies are justified in PLAN: unrestricted exact rational arithmetic,
RFC 8785 numeric formatting and SHA-256. All target and importer code is independent.
