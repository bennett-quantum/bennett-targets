#!/usr/bin/env python3
"""Independent CLI-only consumer checks. Exit 2 means known blocked acceptance."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
from fractions import Fraction

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / '.tmp' / 'verify'
WORK.mkdir(parents=True, exist_ok=True)
for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'DBUS_SESSION_BUS_ADDRESS', 'BROWSER'):
    os.environ.pop(key, None)
os.environ['TMPDIR'] = str(ROOT / '.tmp')
VALIDATOR = ROOT / 'validator/target/debug/bennett-targets-validator'
VM = ROOT.parent / 'bennett-vm/target/debug/bennett-vm'
REF = ROOT.parent / 'bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref'
FIXTURES = Path(os.environ.get('BENNETT_TARGETS_E_CASES', ROOT.parent / 'bennett-isa-spec/conformance/cases/E'))
OBJECT = ROOT.parent / 'bennett-isa-spec/conformance/cases/objects/registers.bisa'
REPORT = {'fixture_bytes': 0, 'fixture_hashes': 0, 'vm_agreements': [], 'vm_divergences': [], 'rejection_diagnostics': [], 'reference_unavailable': [], 'model_runs': [], 'importer_runs': [], 'published_unavailable': [], 'partial_checks': []}

# These sixteen scenarios remain executable in the validator's Rust suite-E test.
# An upstream unavailable marker never removes one of these obligations.
PUBLISHED_CASES = {
    'E-aggregation', 'E-ast-division-by-zero', 'E-ast-eps-out-of-range',
    'E-ast-log2inv', 'E-ast-unbound-variable', 'E-epsilon-zero-entry',
    'E-epsilon-zero-exact', 'E-imports', 'E-jcs-target-hash',
    'E-matching-first-entry', 'E-mbu-must-be-false',
    'E-scheme-correctness-negative', 'E-scheme-expansion', 'E-table-index',
    'E-target-staged-probe', 'E-timing-overlap',
}

def classify_published(case):
    name = case['id']
    if name in PUBLISHED_CASES:
        return True
    require('unavailable' in case, 'unknown published case: ' + name)
    block = case['unavailable']
    reason = block.get('reason') if isinstance(block, dict) else None
    require(isinstance(reason, str) and reason.strip(),
            name + ': published unavailable.reason must be non-empty')
    require('b-isa-spec-' in reason or 'b-targets-' in reason,
            name + ': published unavailable.reason must name a bead')
    REPORT['published_unavailable'].append({'id': name, 'reason': reason})
    print('UNAVAILABLE ' + name + ': ' + reason, flush=True)
    return False

def run(binary, *args):
    return subprocess.run([str(binary), *map(str, args)], text=True, capture_output=True, check=False)

def require(condition, message):
    if not condition:
        raise AssertionError(message)

def validate(path):
    return run(VALIDATOR, 'validate', path)

def vm(path, entry='allocation'):
    return run(VM, 'run-program', OBJECT, entry, '--args', '', '--target', path)

def canonical(document, name):
    source = WORK / (name + '.source.json')
    source.write_text(json.dumps(document, ensure_ascii=False))
    result = run(VALIDATOR, 'canon', source)
    require(result.returncode == 0, result.stderr)
    result = json.loads(result.stdout)
    path = WORK / (name + '.json')
    path.write_text(result['canonical'])
    return path

def comparison(name, path):
    ours, other = validate(path), vm(path)
    require(ours.returncode in (0, 1) and other.returncode in (0, 1), name)
    if ours.returncode == other.returncode:
        if ours.returncode:
            require('§' in ours.stderr and '§' in other.stderr, name + ': rule citation absent')
            rules = {
                'E-epsilon-zero-entry':'eps', 'E-mbu-must-be-false':'mbu', 'E-noncanonical':'canonical',
                'mbu':'mbu', 'capacity-fraction':'integer', 'dirty-negative':'nonnegative',
                'rate-variable':'number', 'rate-unknown-class':'class', 'missing-mandatory-resource':'peak',
                'unknown-resource-kind':'time', 'unknown-gate':'gate', 'cost-unknown-resource':'resource',
                'cost-negative':'nonnegative', 'cost-unbound':'theta', 'cost-arity':'requires 2',
                'cost-unknown-operator':'operator', 'cost-divide-zero':'division by zero',
                'epsilon-exact-reference':'eps', 'gate-unknown-class':'class', 'controls-reversed':'range',
                'capability-negative':'nonnegative', 'measurement-not-bool':'bool',
                'objective-unknown-resource':'resource', 'import-unknown-resource':'resource', 'qram-unbound':'unbound k',
            }
            rule = rules.get(name)
            require(rule is not None, 'unknown published case: ' + name)
            require(rule in ours.stderr and rule in other.stderr, name + ': different rejection rule')
            REPORT['rejection_diagnostics'].append({'name':name,'validator':ours.stderr.strip(),'vm':other.stderr.strip()})
        REPORT['vm_agreements'].append(name)
    else:
        raise AssertionError(name + ': consumer divergence: ' + repr((ours.returncode, ours.stderr, other.returncode, other.stderr)))

cases = {}
for path in sorted(FIXTURES.glob('*/case.json')):
    case = json.loads(path.read_text())
    name = case['id']
    require(name == path.parent.name and name not in cases, str(path) + ': duplicate/mismatched id')
    cases[name] = (case, classify_published(case))
require(PUBLISHED_CASES <= cases.keys(), 'missing executable published cases: ' + repr(PUBLISHED_CASES - cases.keys()))
# Catalogue counts; actual scenario execution is checked by validator/tests/suite_e.rs.
REPORT['suite_e_cases'] = {'executable': len(PUBLISHED_CASES),
                    'unavailable': len(REPORT['published_unavailable']), 'total': len(cases)}

for path in sorted(FIXTURES.glob('*/target.json')):
    result = run(VALIDATOR, 'canon', path)
    require(result.returncode == 0, result.stderr)
    result = json.loads(result.stdout)
    require(result['canonical'].encode() == path.read_bytes(), str(path))
    require(result['target_hash'] == hashlib.sha256(path.read_bytes()).hexdigest(), str(path))
    REPORT['fixture_bytes'] += 1
    REPORT['fixture_hashes'] += 1
    case, executable = cases[path.parent.name]
    stated_hash = case.get('expected_outcome', {}).get('target_hash')
    if stated_hash is not None:
        require(result['target_hash'] == stated_hash, str(path) + ': published target_hash mismatch')
    if executable:
        comparison(path.parent.name, path)
    elif path.parent.name == 'E-topology-grid-09':
        ours, other = validate(path), vm(path)
        require(ours.returncode == other.returncode == 1 and not ours.stdout and not other.stdout,
                'topology: changed ISA 0.8 schema behavior; re-evaluate runnable obligations')
        require('§8.3' in ours.stderr and 'unknown field topology' in ours.stderr, ours.stderr)
        require('§8' in other.stderr and 'unknown field topology' in other.stderr, other.stderr)
        REPORT['partial_checks'].append({
            'id': path.parent.name, 'canonical_hash': result['target_hash'],
            'validator_error': ours.stderr.strip(), 'vm_error': other.stderr.strip(),
            'scope': 'bytes/hash and 0.8 schema rejection only; topology semantics unavailable',
            'spec': '§8.3 lists the target fields without topology; §8.2: gates on disjoint wires MAY overlap unless max_parallel or a topology is declared; all-to-all over logical wires.',
        })
    else:
        # Do not silently hide a target that the validator can evaluate.
        ours = validate(path)
        require(ours.returncode == 1 and not ours.stdout and '§' in ours.stderr,
                'unclassified runnable published target: ' + path.parent.name)
        REPORT['partial_checks'].append({'id': path.parent.name, 'validator_error': ours.stderr.strip()})
    other = run(REF, 'run-program', OBJECT, 'allocation', '--args', '', '--target', path)
    require(other.returncode == 1 and not other.stdout and 'staged deliverable 9' in other.stderr and ('§8' in other.stderr or '§6.4' in other.stderr), other.stderr)
    REPORT['reference_unavailable'].append(path.parent.name)
comparison('E-noncanonical', FIXTURES / 'E-target-staged-probe/noncanonical.json')

base = json.loads((FIXTURES / 'E-jcs-target-hash/target.json').read_text())
mutations = {
 'pointer-width-capabilities': ('ptr_widths_supported', [16,32,128]),
 'mbu': ('capabilities.mbu', True),
 'capacity-fraction': ('classes.fast.capacity', 1.5),
 'dirty-negative': ('classes.fast.dirty_rate', -1),
 'rate-variable': ('resources.qubit_second.rate', {'fast': {'var': 'k'}}),
 'rate-unknown-class': ('resources.qubit_second.rate', {'missing': 1}),
 'missing-mandatory-resource': ('resources.peak_qubits.kind', 'count'),
 'unknown-resource-kind': ('resources.time.kind', 'invalid'),
 'unknown-gate': ('gates.0.op', 'invented'),
 'cost-unknown-resource': ('gates.0.cost.missing', 1),
 'cost-negative': ('gates.0.cost.time', -1),
 'cost-unbound': ('gates.0.cost.time', {'var':'theta'}),
 'cost-arity': ('gates.0.cost.time', {'op':'+','args':[1]}),
 'cost-unknown-operator': ('gates.0.cost.time', {'op':'pow','args':[1,2]}),
 'cost-divide-zero': ('gates.0.cost.time', {'op':'/','args':[1,0]}),
 'epsilon-exact-reference': ('gates.0.cost.time', {'var':'eps'}),
 'gate-unknown-class': ('gates.0.classes', ['unknown']),
 'controls-reversed': ('gates.0.controls', {'min':2,'max':1}),
 'capability-negative': ('capabilities.classical_step_time', -1),
 'measurement-not-bool': ('capabilities.measurement.mid_circuit', 1),
 'objective-unknown-resource': ('objective', {'weighted': {'absent':1}}),
 'import-unknown-resource': ('imports', {'m':{'version':'v','map':{'q':['missing',1]}}}),
 'qram-unbound': ('qram', {'load':{'cost':{'time':{'var':'k'}}},'store':{'cost':{'time':1}},'exch':{'cost':{'time':1}}}),
 'canonical-small-exponent': ('capabilities.classical_step_time', 1e-8),
}
for name, (path, value) in mutations.items():
    document = copy.deepcopy(base)
    keys = path.split('.')
    node = document
    for key in keys[:-1]:
        node = node[int(key)] if isinstance(node,list) else node[key]
    key = keys[-1]
    node[int(key) if isinstance(node,list) else key] = value
    comparison(name, canonical(document, name))

for path in sorted((ROOT / 'targets').glob('*.json')):
    require(validate(path).returncode == 0, str(path))
    result = vm(path, 'views')
    require(result.returncode == 0, result.stderr)
    data = json.loads(result.stdout)
    require(data['status'] == 'COMPLETED' and data['target_error'] == '0', str(path))
    require(data['target_resources']['gate_count'] == '2', str(path))
    require(sum(int(v) for v in data['target_resources']['peak_qubits'].values()) == 2, str(path))
    REPORT['model_runs'].append({'name':path.name,'hash':hashlib.sha256(path.read_bytes()).hexdigest(),'resources':data['target_resources']})

for command, filename in [('import-qualtran','qualtran-gate-counts.json'),('import-azure','azure-qubit-params.json')]:
    result = run(VALIDATOR, command, ROOT / 'targets/ideal-all-to-all.json', ROOT / 'importers/fixtures' / filename)
    require(result.returncode == 0, result.stderr)
    imported = json.loads(result.stdout)
    path = WORK / (filename + '.target.json')
    path.write_text(imported['canonical'])
    require(validate(path).returncode == 0, filename)
    result = vm(path,'views')
    require(result.returncode == 0, result.stderr)
    require(json.loads(result.stdout)['status'] == 'COMPLETED', filename)
    REPORT['importer_runs'].append(filename)

# Reproduce the normative inc8 object directly from published §16.3, adding a
# CC wrapper whose interface hash is independently derived from §9.2.
spec = (ROOT.parent / 'bennett-isa-spec/spec/Bennett-ISA-Spec.md').read_text()
start = spec.index('bennett-object isa=0.8', spec.index('### 16.3'))
text = spec[start:spec.index('```', start)]
lines = text.splitlines()
position = lines.index('section symbols') + 1
symbols = json.loads(lines[position])
inc_hash = symbols['0']['interface_hash']
body = ('fn cc @launch (tk:token) () () () ()\n^entry():\n'
        'x = qreg.new any 8\nt = qreg.new any 8\nf = qreg.new any 4\n'
        'oracle_call @inc8 tk (x) t f () ()\n'
        'oracle_call @inc8 tk (x) t f () ()\n'
        'qreg.free f\nqreg.free t\nqreg.free x\nret')
signature = {'abi':'0.8','kind':'cc','params':['token'],'results':[],'tier':'cc'}
envelope = ('isa=0.8\nptr_width=32\n' + json.dumps(signature, sort_keys=True, separators=(',',':'))
            + '\n' + body.replace('@inc8', '@' + inc_hash) + '\n' + inc_hash + '\n')
symbols['1'] = {'interface_hash':hashlib.sha256(envelope.encode()).hexdigest(), 'kind':'cc', 'name':'launch', 'signature':signature}
lines[position] = json.dumps(symbols, sort_keys=True, separators=(',',':'))
launch = WORK / 'spec-inc8-launch.bisa'
launch.write_text('\n'.join(lines) + '\n' + body + '\n')
for binary in (VM, REF):
    result = run(binary, 'check', launch)
    require(result.returncode == 0, result.stderr)
    result = run(binary, 'run-program', launch, 'launch', '--arg', 'token:3,0,0,0')
    require(result.returncode == 0 and json.loads(result.stdout)['status'] == 'COMPLETED', result.stderr)
REPORT['spec_example_target_runs'] = []
for path in sorted((ROOT / 'targets').glob('*.json')):
    result = run(VM, 'run-program', launch, 'launch', '--arg', 'token:3,0,0,0', '--target', path)
    require(result.returncode == 1 and not result.stdout and 'required ref library is unpublished' in result.stderr and '§8/§11' in result.stderr, result.stderr)
    REPORT['spec_example_target_runs'].append({'target':path.name, 'state':'BLOCKED', 'reason':result.stderr.strip()})

# Exact timing/occupancy expectations derived from seven controller instructions,
# two X leaves and two live wires over five controller steps plus both leaves.
expected_times = {'ideal-all-to-all.json':('9','14'), 'superconducting-grid-illustrative.json':('0.00011','0.00018'), 'trapped-ion-illustrative.json':('0.000027','0.00005')}
for model in REPORT['model_runs']:
    time, occupancy = expected_times[model['name']]
    require(Fraction(model['resources']['time']) == Fraction(time), model['name'] + ': makespan')
    require(sum(Fraction(n) for n in model['resources']['qubit_second'].values()) == Fraction(occupancy), model['name'] + ': occupancy')

REPORT['reference_semantic_agreements'] = 0
REPORT['circuits_target_cli'] = 'unavailable: documented CLI exposes no target input'
REPORT['acceptance_blockers'] = [
    {'id': 'b-targets-dd3', 'reason': 'Topology semantics: §8.2 mentions topology but §8.3 publishes no topology/placement schema; E-topology-grid-09 requires accepted 0.9 (b-isa-spec-wr7).'},
    {'id': 'b-targets-w8o', 'reason': 'eps_decade: §8.1 names this index without defining decade selection in accepted ISA 0.8 (b-isa-spec-f31).'},
    {'id': 'b-targets-7dv', 'reason': 'Unpublished ref expansion library: all three target-backed normative inc8 runs refuse lift.const.i8 expansion (§8/§11; b-isa-spec-8r4 / bennett-vm-47t); E-ref-09-qxor-witness is not the complete catalog.'},
    {'id': 'b-targets-38r', 'reason': 'Reference CLI surface: every target attempt is refused as staged deliverable 9; zero semantic target agreements. Circuits also documents no target CLI.'},
]
REPORT['acceptance'] = 'BLOCKED: topology semantics, eps_decade, unpublished ref expansion library, reference target CLI surface'
require(len(REPORT['vm_agreements']) == 31, 'VM agreement baseline changed; deliberate rebaseline required')
require(REPORT['fixture_bytes'] == REPORT['fixture_hashes'] == 7,
        'canonical fixture baseline changed; deliberate rebaseline required')
REPORT['binary_sha256'] = {str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in (VM, REF)}
(ROOT / 'docs/verification.json').write_text(json.dumps(REPORT, indent=2) + '\n')
print(json.dumps({k:v for k,v in REPORT.items() if k != 'rejection_diagnostics'}, indent=2))
# Known missing contract/consumer surfaces are never counted as successful completion.
sys.exit(2)
