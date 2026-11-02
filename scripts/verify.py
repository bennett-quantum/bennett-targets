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
VM_TOPOLOGY_BEAD = 'b-targets-dt2'
WORK = ROOT / '.tmp' / 'verify'
WORK.mkdir(parents=True, exist_ok=True)
for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'DBUS_SESSION_BUS_ADDRESS', 'BROWSER'):
    os.environ.pop(key, None)
os.environ['TMPDIR'] = str(ROOT / '.tmp')
VALIDATOR = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'validator/target')) / 'debug/bennett-targets-validator'
VM = ROOT.parent / 'bennett-vm/target/debug/bennett-vm'
REF = ROOT.parent / 'bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref'
FIXTURES = Path(os.environ.get('BENNETT_TARGETS_E_CASES', ROOT.parent / 'bennett-isa-spec/conformance/cases/E'))
OBJECT = ROOT.parent / 'bennett-isa-spec/conformance/cases/objects/registers.bisa'
REPORT = {'spec': 'ISA 0.9 §8 (target 0.9)', 'consumer_gaps': [], 'fixture_bytes': 0, 'fixture_hashes': 0, 'vm_agreements': [], 'vm_divergences': [], 'rejection_diagnostics': [], 'reference_unavailable': [], 'model_runs': [], 'importer_runs': [], 'published_unavailable': [], 'partial_checks': []}

# These seventeen scenarios remain executable in the validator's Rust suite-E test.
# An upstream unavailable marker never removes one of these obligations.
PUBLISHED_CASES = {
    'E-aggregation', 'E-ast-division-by-zero', 'E-ast-eps-out-of-range',
    'E-ast-log2inv', 'E-ast-unbound-variable', 'E-epsilon-zero-entry',
    'E-epsilon-zero-exact', 'E-imports', 'E-jcs-target-hash',
    'E-matching-first-entry', 'E-mbu-must-be-false',
    'E-scheme-correctness-negative', 'E-scheme-expansion', 'E-table-index',
    'E-target-staged-probe', 'E-timing-overlap', 'E-topology-grid-09',
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
                'eps-decade-exact-entry':'eps', 'objective-unknown-resource':'resource', 'import-unknown-resource':'resource', 'qram-unbound':'unbound k',
            }
            rule = rules.get(name)
            require(rule is not None, 'unknown published case: ' + name)
            require(rule in ours.stderr and rule in other.stderr, name + ': different rejection rule')
            REPORT['rejection_diagnostics'].append({'name':name,'validator':ours.stderr.strip(),'vm':other.stderr.strip()})
        REPORT['vm_agreements'].append(name)
    else:
        raise AssertionError(name + ': consumer divergence: ' + repr((ours.returncode, ours.stderr, other.returncode, other.stderr)))

def query(command, target, body, name):
    source = WORK / (name + '.query.json')
    source.write_text(json.dumps(body))
    return run(VALIDATOR, command, target, source)

def vm_topology_gap(path, entry='allocation'):
    """The VM's refusal of a valid 0.9 target is a recorded consumer gap, not agreement."""
    other = vm(path, entry)
    if other.returncode == 0:
        return other
    require('unknown field topology' in other.stderr, path.name + ': unexpected VM error: ' + other.stderr)
    REPORT['consumer_gaps'].append({'consumer': 'bennett-vm', 'target': path.name, 'bead': VM_TOPOLOGY_BEAD, 'error': other.stderr.strip()})
    return None

def check_topology(path, case):
    scenario, expected = case['scenario'], case['expected_outcome']
    require(validate(path).returncode == 0, 'topology witness must load under §8.3')
    yes = query('adjacency', path, {'vertices': scenario['adjacent']}, 'topology-adjacent')
    require(yes.returncode == 0 and json.loads(yes.stdout)['realisable'] is True, yes.stderr)
    no = query('adjacency', path, {'vertices': scenario['non_adjacent']}, 'topology-non-adjacent')
    require(no.returncode == 1 and not no.stdout and expected['native_cx_non_adjacent'] in no.stderr, no.stderr)
    document = json.loads(path.read_text())
    del document['topology']
    absent = query('adjacency', canonical(document, 'topology-absent'), {'vertices': scenario['non_adjacent']}, 'topology-absent')
    require(absent.returncode == 0, 'absent topology must be all-to-all: ' + absent.stderr)
    REPORT['validator_topology'] = {'adjacent': 'realisable', 'non_adjacent': no.stderr.strip(), 'absent': 'all-to-all'}
    vm_topology_gap(path)

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
    if path.parent.name == 'E-topology-grid-09':
        check_topology(path, case)
    elif executable:
        comparison(path.parent.name, path)
    else:
        # Do not silently hide a target that the validator can evaluate.
        ours = validate(path)
        require(ours.returncode == 1 and not ours.stdout and '§' in ours.stderr,
                'unclassified runnable published target: ' + path.parent.name)
        REPORT['partial_checks'].append({'id': path.parent.name, 'validator_error': ours.stderr.strip()})
    other = run(REF, 'run-program', OBJECT, 'allocation', '--args', '', '--target', path)
    require(other.returncode == 1 and not other.stdout and 'staged deliverable 9' in other.stderr and ('§8' in other.stderr or '§6.4' in other.stderr), other.stderr)
    REPORT['reference_unavailable'].append(path.parent.name)
# §8.1 EPS-DECADE witness, parsed from the published text rather than copied.
spec09 = (ROOT.parent / 'bennett-isa-spec/spec/Bennett-ISA-Spec-0.9.md').read_text()
fence = spec09.index('```json', spec09.index('**EPS-DECADE witness**')) + len('```json')
witness = json.loads(spec09[fence:spec09.index('```', fence)])
require(len(witness) == 6, 'EPS-DECADE witness changed')
decade = json.loads((FIXTURES / 'E-jcs-target-hash/target.json').read_text())
decade['tables'] = {'decade': list(range(5))}
decade_target = canonical(decade, 'eps-decade')
lookup = {'table': 'decade', 'index': 'eps_decade'}
for row in witness:
    out = query('eval', decade_target, {'expr': lookup, 'bindings': {'eps': row['eps']}}, 'eps-decade')
    require(out.returncode == 0 and json.loads(out.stdout)['value'] == str(row['index']), row['eps'] + out.stderr)
for bad in ('0', '1.5', None):
    out = query('eval', decade_target, {'expr': lookup, 'bindings': {} if bad is None else {'eps': bad}}, 'eps-decade')
    require(out.returncode == 1 and '§8.1' in out.stderr, repr(bad) + out.stderr)
REPORT['eps_decade_witness'] = len(witness)

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
 'eps-decade-exact-entry': ('gates.0.cost.time', {'table':'t','index':'eps_decade'}),
}
for name, (path, value) in mutations.items():
    document = copy.deepcopy(base)
    keys = path.split('.')
    node = document
    for key in keys[:-1]:
        node = node[int(key)] if isinstance(node,list) else node[key]
    key = keys[-1]
    node[int(key) if isinstance(node,list) else key] = value
    if name.startswith('eps-decade'):
        document['tables'] = {'t': [1, 2, 3, 4, 5]}
    comparison(name, canonical(document, name))
valid_decade = copy.deepcopy(base)
valid_decade['tables'] = {'t': [1, 2, 3, 4, 5]}
valid_decade['gates'][0]['epsilon'] = 0.001
valid_decade['gates'][0]['cost']['time'] = {'table':'t','index':'eps_decade'}
comparison('eps-decade-valid', canonical(valid_decade, 'eps-decade-valid'))

for path in sorted((ROOT / 'targets').glob('*.json')):
    require(validate(path).returncode == 0, str(path))
    result = vm_topology_gap(path, 'views')
    if result is None:
        continue
    data = json.loads(result.stdout)
    require(data['status'] == 'COMPLETED' and data['target_error'] == '0', str(path))
    require(data['target_resources']['gate_count'] == '2', str(path))
    require(sum(int(v) for v in data['target_resources']['peak_qubits'].values()) == 2, str(path))
    REPORT['model_runs'].append({'name':path.name,'hash':hashlib.sha256(path.read_bytes()).hexdigest(),'resources':data['target_resources']})

for command, filename in [('import-qualtran','qualtran-gate-counts.json'),('import-azure','azure-qubit-params.json'),('import-azure','azure-nonzero-error.json'),('import-azure','azure-majorana.json')]:
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

# Reproduce the normative inc8 object directly from published 0.9 §16.3, adding a
# CC wrapper whose interface hash is independently derived from §9.2.
spec = spec09
start = spec.index('bennett-object isa=0.9', spec.index('### 16.3'))
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
signature = {'abi':'0.9','kind':'cc','params':['token'],'results':[],'tier':'cc'}
envelope = ('isa=0.9\nptr_width=32\n' + json.dumps(signature, sort_keys=True, separators=(',',':'))
            + '\n' + body.replace('@inc8', '@' + inc_hash) + '\n' + inc_hash + '\n')
symbols['1'] = {'interface_hash':hashlib.sha256(envelope.encode()).hexdigest(), 'kind':'cc', 'name':'launch', 'signature':signature}
lines[position] = json.dumps(symbols, sort_keys=True, separators=(',',':'))
launch = WORK / 'spec-inc8-launch-09.bisa'
launch.write_text('\n'.join(lines) + '\n' + body + '\n')
REPORT['spec_example_untargeted'] = []
for label, binary in (('vm', VM), ('reference', REF)):
    checked = run(binary, 'check', launch)
    ran = run(binary, 'run-program', launch, 'launch', '--arg', 'token:3,0,0,0') if checked.returncode == 0 else None
    REPORT['spec_example_untargeted'].append({'consumer': label, 'check': checked.returncode,
        'run': None if ran is None else ran.returncode,
        'error': (checked.stderr if ran is None else ran.stderr).strip()[:400]})

# §11.1: recompute the published library hash, then the gates the inc8 rows need.
library_bytes = (ROOT.parent / 'bennett-isa-spec/spec/ref-0.9.json').read_bytes()
library_hash = hashlib.sha256(b'isa=0.9\nexpansion=ref\nversion=0.9\n' + library_bytes + b'\n').hexdigest()
require(library_hash in spec09, 'ref-0.9.json envelope hash is not the one bound in §11.1: ' + library_hash)
REPORT['ref_library_hash'] = library_hash
library = json.loads(library_bytes)
recipes = {}
for recipe in library['recipes']:
    recipes.setdefault(recipe['kind'], []).append(recipe)
def walk(node, gates, invoked):
    if isinstance(node, dict):
        if 'gate' in node:
            gates.add(node['gate'])
        if 'invoke' in node:
            invoked.add(node['invoke'])
        for value in node.values():
            walk(value, gates, invoked)
    elif isinstance(node, list):
        for value in node:
            walk(value, gates, invoked)
inc8_gates, todo, seen = set(), ['lift.const', 'lift.add', 'lift.retval'], set()
while todo:
    kind = todo.pop()
    if kind not in seen:
        seen.add(kind)
        require(kind in recipes, '§11.1 catalog has no recipe ' + kind)
        for recipe in recipes[kind]:
            invoked = set()
            walk(recipe.get('body', recipe.get('operations')), inc8_gates, invoked)
            todo.extend(invoked)
require(inc8_gates, 'inc8 recipe closure emitted no gates')
REPORT['ref_inc8_rows'] = {'recipes': sorted(seen), 'gates': sorted(inc8_gates)}
for path in sorted((ROOT / 'targets').glob('*.json')):
    model = json.loads(path.read_text())
    available = {g['op'] for g in model['gates']} | {s['realises']['op'] for s in model['schemes']}
    require(inc8_gates <= available, path.name + ': no entry for inc8 row gates ' + repr(inc8_gates - available))
REPORT['spec_example_target_runs'] = []
for path in sorted((ROOT / 'targets').glob('*.json')):
    result = run(VM, 'run-program', launch, 'launch', '--arg', 'token:3,0,0,0', '--target', path)
    # Refusal is recorded, never a pass; only a COMPLETED run could close b-targets-7dv.
    completed = result.returncode == 0 and json.loads(result.stdout).get('status') == 'COMPLETED'
    REPORT['spec_example_target_runs'].append({'target': path.name,
        'state': 'COMPLETED' if completed else 'BLOCKED',
        'reason': '' if completed else (result.stderr or result.stdout).strip()[:400]})
inc8_blocked = [r for r in REPORT['spec_example_target_runs'] if r['state'] != 'COMPLETED']

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
    {'id': 'b-targets-38r', 'reason': 'Reference CLI surface: every target attempt is refused as staged deliverable 9; zero semantic target agreements (b-isa-spec-xg3). Circuits also documents no target CLI.'},
]
if inc8_blocked:
    REPORT['acceptance_blockers'].append({'id': 'b-targets-7dv', 'reason': 'Target-backed normative inc8 does not complete: ' + inc8_blocked[0]['reason']})
if REPORT['consumer_gaps']:
    REPORT['acceptance_blockers'].append({'id': VM_TOPOLOGY_BEAD, 'reason': 'Validator implements §8.3 topology; the VM refuses it, so no consumer agrees yet: ' + REPORT['consumer_gaps'][0]['error']})
REPORT['acceptance'] = 'BLOCKED: ' + ', '.join(b['id'] for b in REPORT['acceptance_blockers'])
require(len(REPORT['vm_agreements']) == 33, 'VM agreement baseline changed; deliberate rebaseline required')
require(REPORT['fixture_bytes'] == REPORT['fixture_hashes'] == 7,
        'canonical fixture baseline changed; deliberate rebaseline required')
REPORT['binary_sha256'] = {str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in (VM, REF)}
(ROOT / 'docs/verification.json').write_text(json.dumps(REPORT, indent=2) + '\n')
print(json.dumps({k:v for k,v in REPORT.items() if k != 'rejection_diagnostics'}, indent=2))
# Repo gate (advisor ruling, 2026-11-02): exit 0 means this repo's own checks passed; a failed check
# has already raised through require(). Milestone status is reported, never an exit code: acceptance
# stays BLOCKED until the blockers below close, and this is not a claim of successful completion.
print('MILESTONE STATUS: ' + REPORT['acceptance'] + ' (repo gate passed; acceptance not claimed)')
for blocker in REPORT['acceptance_blockers']:
    print('  ' + blocker['id'] + ': ' + blocker['reason'])
