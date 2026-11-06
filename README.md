# bennett-targets

Independent Bennett ISA 0.9 §8 (target 0.9) target validator, canonical descriptions and first
Qualtran/Azure adapters. See [PLAN](docs/PLAN.md),
[models](targets/README.md), and [field mappings](importers/README.md).

[Ecosystem handoff](../bennett/HANDOFF.md).

Run `scripts/pre-push` from this checkout for the pinned consumer gate. For the
fast default validator checks:

```sh
cargo test --offline --manifest-path validator/Cargo.toml -j3 -- --test-threads=4
python3 -m unittest discover -s scripts -p 'test_*.py'
```

The local gate unsets desktop/browser variables and uses this repo's `.tmp/`.
Build via Cargo with `--manifest-path validator/Cargo.toml`. The binary is
`${CARGO_TARGET_DIR:-validator/target}/debug/bennett-targets-validator`:

- `validate <file>`: canonical input and schema/context checks, JSON valid/hash.
- `canon <file>`: strict JSON to canonical bytes/hash encoded in a JSON result;
  canonicalization does not assert semantic validity (negative fixtures work).
- `eval <target> <query>`: `{"expr":..,"bindings":{"eps":"0.1"}}` to an exact rational.
- `adjacency <target> <query>`: `{"vertices":[u,v]}` to realisable, or
  `E_SCHEDULE_INFEASIBLE` naming §17 item 15 (§8.3 topology).
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

Acceptance is **blocked**, not complete: the validator implements K9 eps_decade
and K14 topology, but the VM refuses topology targets (b-targets-dt2), the VM
does not yet execute the normative inc8 example under a target (b-targets-7dv) and
the reference CLI refuses all targets (b-targets-38r); evidence is in
[verification](docs/verification.json). The consumer harness returns exit 0 when
repo checks pass and reports the outstanding acceptance blockers separately. It
fails immediately on missing binaries, broken
witnesses or unexpected run results. Circuits' documented CLI does not yet accept
targets. `SPEC_VERSIONS.toml` claims 0.9 for §8 only: the objects the harness runs
under a target are the 0.8 registers fixture, and the 0.9 inc8 runs untargeted.
QEC and hierarchy import await b-targets-gyp / b-isa-spec-fd0.

Small dependencies are justified in PLAN: unrestricted exact rational arithmetic,
RFC 8785 numeric formatting and SHA-256. All target and importer code is independent.

Tests and consumer checks read sibling inputs only from the `gate.toml` cache;
a missing pin fails loudly. [Quality measurements](docs/quality.md) record timing,
coverage and the remaining consumer/contract limitations. Numeric provenance is
checked by JSON path and exact rational value for all four models. Real Qualtran
and Azure sample provenance and optional capture instructions are in the
[adapter documentation](importers/README.md).
