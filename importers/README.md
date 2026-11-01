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
flattened. Complete symbolic/hierarchical models remain in PLAN, b-targets-gyp and b-isa-spec-fd0.
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
| instructionSet | GateBased or Majorana; each admits only its own fields; others rejected |
| oneQubitGateTime | GateBased: seconds quantity and x/h/s/sdg gate-time overlays |
| twoQubitGateTime | GateBased: seconds quantity and cx/cz/swap overlays; defaults to oneQubitGateTime |
| tGateTime | seconds quantity and t/tdg overlays; GateBased defaults to oneQubitGateTime, Majorana requires it |
| oneQubitMeasurementTime | seconds quantity only; no runtime duration inferred |
| twoQubitJointMeasurementTime | Majorana: seconds quantity only; defaults to oneQubitMeasurementTime |
| oneQubitGateErrorRate | GateBased: count resource, charged on x/h/s/sdg |
| twoQubitGateErrorRate | GateBased: count resource, charged on cx/cz/swap; defaults to oneQubitGateErrorRate |
| tGateErrorRate | count resource, charged on t/tdg; GateBased defaults to oneQubitGateErrorRate, Majorana requires it |
| oneQubitMeasurementErrorRate | number or {process, readout}; quantity only, no charged event |
| twoQubitJointMeasurementErrorRate | Majorana: as above; defaults to oneQubitMeasurementErrorRate |
| idleErrorRate | as above; defaults to oneQubitMeasurementErrorRate |

All resolved defaults are explicit in the mapping report. Durations accept s, ms,
us, µs, μs, ns and ps, convert exactly to seconds, and map to separate
`azure.<field>.seconds` count resources in imports, factor 1. The base must use
seconds for its time resource. Missing mandatory inputs and unknown units are errors.

**Failure probabilities are named resources, never epsilon.** North Star §7 says
imported models become "new named resources" where units do not align. Each rate
is a §8.2 `count` resource with unit `failure_probability` (`azure.<field>`, or
`azure.<field>.process` / `.readout`), in [0, 1]. On gates it sums per leaf event,
which is a first-order accounting dimension: neither ISA §8.1's phase-sensitive
operator-norm approximation error nor a composed failure probability. `epsilon`
is never changed. Measurement and idle rates are quantities only, because §8 has
no such charged event. A Majorana import keeps the base's gates; joint measurements
are not gates and none is invented.

`qecScheme`, `errorBudget`, `constraints`, `distillationUnitSpecifications`,
`profiling`, `estimateType`, `resumeAfterFailedIteration`, batch/items/result
objects and every other unknown field are explicit errors, not dropped scope.
QEC/code-distance formulae, factory throughput and hierarchy need a contract that
§8 does not have: b-targets-gyp, with exact proposed spec text in b-isa-spec-fd0.

The hand-written fixtures are illustrative, with all fields covered by tests;
they are executable translation witnesses, not physical hardware.
