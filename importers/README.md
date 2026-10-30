# Resource-model adapters

CLI: `bennett-targets-validator import-qualtran <base-target> <input-json>` and
`import-azure <base-target> <input-json>`. All paths in this project are absolute.
The result is JSON with `canonical`, `target_hash`, `mapping`, `quantities`,
and `source`. Write `canonical` with **no LF** to obtain the target file.
The explicit base supplies classes, objective, executable gate definitions and
capabilities: a count report alone cannot imply those. Existing import/resource
names collide loudly; unrelated base fields retain their original semantics.
Every key in an input must be recognized; unknown fields are errors, at any level.
No Python package installation, cloud job, or code import is required.

## Qualtran

Input is the concrete JSON serialization of `GateCounts.asdict()` documented by
[Qualtran](https://qualtran.readthedocs.io/en/latest/reference/qualtran/resource_counting/GateCounts.html)
(accessed 2026-10-30). All seven named quantities remain distinct. Missing
fields use GateCounts' zero defaults and the mapping report marks them defaulted.

| Source field | Target resource and imports map | Factor |
| --- | --- | --- |
| t | qualtran.t (count, unit t) | 1 |
| toffoli | qualtran.toffoli (count, unit toffoli) | 1 |
| cswap | qualtran.cswap (count, unit cswap) | 1 |
| and_bloq | qualtran.and_bloq (count, unit and_bloq) | 1 |
| clifford | qualtran.clifford (count, unit clifford) | 1 |
| rotation | qualtran.rotation (count, unit rotation) | 1 |
| measurement | qualtran.measurement (count, unit measurement) | 1 |

Counts above 2^53−1 are decimal strings in the quantity report; no host-word
ceiling is imposed on counts. No Toffoli-to-T or rotation synthesis factor is assumed. Symbolic values,
fractional counts and negative counts are explicit errors. AlgorithmSummary,
PhysicalCostModel, factory/QEC objects, protobuf Bloq programs, custom attributes,
and all other source keys are explicitly unsupported inputs, not silently
flattened. Complete symbolic/hierarchical models remain in PLAN and b-targets-gyp.
The adapter adds accounting dimensions and converts quantities; it does not
claim a GateCounts report is a native hardware gate table.

## Azure

Input is the legacy Azure estimator `{"qubitParams": {...}}` JSON, not the
new QDK architecture API. The documented legacy fields are available in the
[Microsoft reference](https://learn.microsoft.com/pl-pl/azure/quantum/overview-resources-estimator)
(accessed 2026-10-30; English URL now redirects to the redesigned estimator).
Source schemas are identified by descriptive adapter versions, not an invented
upstream package version. Original source data is retained as `source.input_json` text to avoid rounding
large integers when the outer report is serialized as JCS.

| Source field | Mapping or explicit error |
| --- | --- |
| qubitParams | exactly one nested hardware parameter object |
| name | source provenance; explicit string required; no implicit preset lookup |
| instructionSet | assert GateBased; Majorana/other values rejected |
| oneQubitGateTime | seconds quantity and existing x/h/s/sdg gate-time overlays |
| twoQubitGateTime | seconds quantity and existing cx/cz/swap gate-time overlays |
| tGateTime | seconds quantity and existing t/tdg gate-time overlays |
| oneQubitMeasurementTime | seconds quantity only; no runtime duration inferred |
| oneQubitMeasurementErrorRate | assert zero; nonzero rejected with field path |
| oneQubitGateErrorRate | assert zero; nonzero rejected with field path |
| twoQubitGateErrorRate | assert zero; defaults to oneQubitGateErrorRate |
| tGateErrorRate | assert zero; defaults to oneQubitGateErrorRate |
| idleErrorRate | assert zero; defaults to oneQubitMeasurementErrorRate |

The last two gate times default to oneQubitGateTime. All resolved defaults are
explicit in the mapping report. Durations accept s, ms, us, µs, μs, ns and ps,
convert exactly to seconds, and map to separate `azure.<field>.seconds` count
resources in imports, factor 1 (source quantities in the report are already
normalized to seconds). The source duration and its unit remain in provenance.
The base must use seconds for its time resource. Missing mandatory inputs and
unknown units are errors. These overlays do not invent missing native operations.

Physical failure probability is not ISA §8.1's phase-sensitive operator-norm
approximation error. Nonzero probabilities and process/readout error objects
are explicit errors, never set to zero or copied into epsilon. The source's
measurement duration is retained as hardware data; §8 supplies no distinct
measurement-operation duration field. Full behavioral measurement import is
blocked by b-targets-gyp. Base classical step/feedforward times remain explicit.

`qecScheme`, `errorBudget`, `constraints`, `distillationUnitSpecifications`,
`profiling`, `estimateType`, `resumeAfterFailedIteration`, batch/items/result
objects and every other top-level field are explicit errors. Within qubitParams,
`twoQubitJointMeasurementTime`, `twoQubitJointMeasurementErrorRate`, custom
expressions and every other unknown field are explicit errors. QEC/code-distance
formulae, factory throughput, hierarchy and stochastic channels require published
bindings/semantics (b-targets-gyp), retained in the full remit rather than dropped.

The hand-written fixtures are illustrative, with all fields covered by tests;
zero-error Azure is an executable translation witness, not physical hardware.
