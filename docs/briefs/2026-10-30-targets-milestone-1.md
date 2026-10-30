# Brief: bennett-targets milestone 1 (validator, canonical descriptions, first vendor models, importers)

Repository `/home/tobias/Projects/bennett-workspace/bennett-targets` (skeleton: `validator/` crate, `targets/`). You are
the sole implementer. It consumes ISA 0.8 only through the published spec `../bennett-isa-spec/spec/Bennett-ISA-Spec.md`
§8 (target 0.8: §8.1 format and expressions, §8.2 resources/aggregation/timing, §8.3 classes/gates/schemes/capabilities,
§8.4 space budget) and never copies code across the contract boundary (CLAUDE.md rule 5). Two consumers of target
descriptions already exist and are your differential oracles through their CLIs only: the reference interpreter
(`../bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref`, README "Commands") and bennett-vm
(`../bennett-vm`, its CLI documented in its README; it implements §8 target evaluation, gate/scheme execution and class
capacities as of commit 849f7f7, and its docs/PROGRESS.md lists the §8 gaps it found: `eps_decade` undefined (ISA
bead cxg / b-isa-spec-f31), canonical target bytes with no trailing LF (clh / b-isa-spec-1pl, fixed in the conformance
fixtures)). The conformance crate `../bennett-isa-spec/conformance` (read-only) holds the corrected suite-E target
fixtures and their expected hashes: your validator must reproduce every one of them.

## Read first, in full
`CLAUDE.md`, `README.md`, `HANDOFF.md`, `SPEC_VERSIONS.toml`; ISA §8 in full plus §9 (number formats, decimals),
§16 (envelope/hash rule); `../bennett/docs/Bennett-NorthStar.md` §4–§5 and §7 (what vendors must be able to express,
and the resource-model landscape the importers target); the VM's `src/target.rs` behaviour as observed through its CLI
(not its code); the conformance crate's suite-E cases and `conformance/PROGRESS.md`.

## Milestone 1
1. `docs/PLAN.md`: scope for the whole CLAUDE.md remit (validator, canonical JCS descriptions per hardware model,
   importers for Qualtran and Azure resource-model formats), milestones, milestone-1 contents.
2. Validator crate: parse, validate against every §8 rule (expressions, resources, aggregation, timing, classes, gates,
   schemes, capabilities, space budget), canonicalise to the exact JCS bytes §8.1 requires, compute the target hash by
   the spec's envelope rule; `validate <file>` and `canon <file>` CLI commands with JSON output; diagnostics cite the
   §8 sentence. Acceptance: byte-identical canonical form and identical hash to every conformance suite-E target
   fixture; agreement with `bennett-isa-ref`/VM on every target they accept or reject, with the same rule.
3. First vendor-neutral descriptions under `targets/`: at least (a) an ideal all-to-all model, (b) a 2D-grid
   superconducting-style model with classes/capacities/timings, (c) a trapped-ion-style model, each hand-derived with
   every number's provenance in a sidecar `.provenance.md` (a public datasheet or paper, or "illustrative"), and all
   three validated and consumed by the VM (`run-program`/target evaluation on a spec example) and by bennett-circuits
   placement if its CLI accepts a target by then (`../bennett-circuits`, read-only).
4. Importers: Qualtran and Azure resource-model formats → §8 descriptions; red-green on small hand-written inputs with
   the mapping of every field stated (and every field §8 cannot express listed as an explicit error, never dropped
   silently). Where §8 lacks a capability the north star §7 landscape needs, `bd create` here (label isa-0.9) quoting
   the sentence; never narrow.

Red-green per feature; fail fast and loud; std-only Rust unless a crate is truly needed (say so).

## Process
Headless (rule 11): `DISPLAY`, `WAYLAND_DISPLAY`, `DBUS_SESSION_BUS_ADDRESS`, `BROWSER` unset, `TMPDIR` under this
repo's `.tmp/`. `bd init` if `.beads/` is absent, milestone epic + tasks, `bd export -o .beads/issues.jsonl` before
committing `.beads/`. Keep `docs/PROGRESS.md` current. Commit regularly: `git -C
/home/tobias/Projects/bennett-workspace/bennett-targets commit -- <paths>` (retry on index.lock; BEADS_HOOK_TIMEOUT=20);
do not push; absolute paths only; never `cd` inside compound commands; never edit any other repository. Final message:
commit hash, PLAN summary, fixture agreement counts (canonical bytes and hashes), consumer agreement counts, the three
descriptions and their provenance, importer field coverage, every §8 gap filed.
