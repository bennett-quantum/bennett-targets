#!/usr/bin/env python3
"""Measure tracked Rust/Python structure and reject regressions against docs/quality.md."""
import argparse
import ast
from itertools import zip_longest
import json
from pathlib import Path
import re
import subprocess
import sys


BASELINE = "<!-- quality-baseline -->\n```json\n"
PROCESS = re.compile(
    r"\bb-(?:isa-spec|dialect-spec|targets|vm|sched|driver|estimate|conformance|arith|circuits|runtimes|plugin)-[a-z0-9]+(?:-[a-z0-9]+)*\b"
    r"|\bjob/[\w-]+|\bquality-(?:targets|isaref|vm|sched|plugin|driver|checker|estimate|runtimes|arith|dialect|circuits|meta)\b"
    r"|\b(?:shift|round|session)[ _-]+\d+", re.I,
)
LITTER = re.compile(r"(?:^|/)(?:__pycache__|PROGRESS[^/]*|LAST-RUN[^/]*|LOCK|\.lock)(?:/|$)|\.(?:pyc|log|darc|lock)$")
RUST_TOKEN = re.compile(
    r'(?P<space>\s+)|(?P<line>//[^\n]*)|(?P<block>/\*)'
    r'|(?P<literal>r(?P<hashes>\#*)"[\s\S]*?"(?P=hashes)|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\\n])\')'
    r'|[A-Za-z_][A-Za-z_0-9]*|&&|\|\||[^\s]', re.S,
)
RUST_BRANCHES = {"if", "for", "while", "loop", "match"}
PY_FUNCTIONS = (ast.FunctionDef, ast.AsyncFunctionDef)
PY_BRANCHES = (ast.If, ast.IfExp, ast.For, ast.AsyncFor, ast.While, ast.ExceptHandler, ast.Match, ast.comprehension)


def rust_tokens(source):
    tokens, pos = [], 0
    while pos < len(source):
        token = RUST_TOKEN.match(source, pos)
        if token is None:
            raise ValueError(f"cannot tokenize Rust at offset {pos}")
        pos = token.end()
        if token.lastgroup == "block":
            depth = 1
            while depth:
                delimiter = re.search(r"/\*|\*/", source[pos:])
                if delimiter is None:
                    raise ValueError("unterminated Rust block comment")
                depth += 1 if delimiter.group() == "/*" else -1
                pos += delimiter.end()
        elif token.lastgroup not in {"space", "line"}:
            tokens.append(("literal" if token.lastgroup == "literal" else token.group(), token.start()))
    return tokens


def rust_metrics(source):
    tokens = rust_tokens(source)
    values = [value for value, _ in tokens]
    stack, closing = [], {}
    for i, value in enumerate(values):
        if value == "{":
            stack.append(i)
        elif value == "}":
            closing[stack.pop()] = i
    if stack:
        raise ValueError("unclosed Rust brace")
    functions = {}
    for i, value in enumerate(values):
        if value == "fn":
            body = i + 1
            while values[body] not in {"{", ";"}:
                body += 1
            if values[body] == "{":
                functions[i] = (body, closing[body])
    lengths, complexity, wrappers = [], 0, 0
    for start, (body, end) in functions.items():
        lengths.append(source.count("\n", tokens[start][1], tokens[end][1]) + 1)
        depth, i = 0, body + 1
        while i < end:
            value = values[i]
            if i in functions:
                i = functions[i][1] + 1
                continue
            if value in RUST_BRANCHES:
                complexity += 1 + depth
            elif value in {"&&", "||", "?"}:
                complexity += 1
            depth += (value == "{") - (value == "}")
            i += 1
        # A single forwarding call; nested calls, computations and blocks are excluded.
        statement = " ".join(values[body + 1:end])
        wrappers += bool(re.fullmatch(
            r"(?:return )?[\w: .]+ \( (?:[\w ,&:]* )?\)(?: \?)?(?: ;)?", statement
        ))
    return {"lengths": sorted(lengths, reverse=True), "complexity": complexity, "wrappers": wrappers}


def python_complexity(node, depth=0):
    if isinstance(node, PY_FUNCTIONS + (ast.Lambda,)):
        return 0
    branch = isinstance(node, PY_BRANCHES)
    score = 1 + depth if branch else 0
    if isinstance(node, ast.BoolOp):
        score += len(node.values) - 1
    return score + sum(python_complexity(child, depth + branch) for child in ast.iter_child_nodes(node))


def python_metrics(source):
    tree = ast.parse(source)
    functions = [node for node in ast.walk(tree) if isinstance(node, PY_FUNCTIONS)]
    wrappers = 0
    for function in functions:
        if len(function.body) == 1 and isinstance(function.body[0], (ast.Return, ast.Expr)):
            value = function.body[0].value
            wrappers += isinstance(value, ast.Call) and all(isinstance(arg, ast.Name) for arg in value.args) and not value.keywords
    return {
        "lengths": sorted((node.end_lineno - node.lineno + 1 for node in functions), reverse=True),
        "complexity": python_complexity(tree) + sum(python_complexity(statement) for node in functions for statement in node.body),
        "wrappers": wrappers,
    }


def measure(root):
    paths = subprocess.check_output(["git", "-C", str(root), "ls-files", "-z"], text=True).split("\0")
    report = {"files": {}, "litter": [], "process_identifiers": {}}
    for name in sorted(set(paths) - {""}):
        path = Path(name)
        if LITTER.search(name) and path.name != "Cargo.lock":
            report["litter"].append(name)
        if path.suffix not in {".rs", ".py", ".json", ".sh"} or name.startswith(".beads/"):
            continue
        source = (root / path).read_text()
        if path.suffix in {".rs", ".py"}:
            try:
                report["files"][name] = (rust_metrics if path.suffix == ".rs" else python_metrics)(source)
            except (ValueError, SyntaxError, IndexError) as error:
                raise ValueError(f"{name}: {error}") from error
        test = "tests" in path.parts or path.name.startswith("test_")
        product = source.split("#[cfg(test)]", 1)[0] if path.suffix == ".rs" else source
        if not test and (name.startswith(("validator/", "scripts/", "targets/", "importers/")) or path.suffix != ".json"):
            count = len(PROCESS.findall(product)) + len(PROCESS.findall(name))
            if count:
                report["process_identifiers"][name] = count
    return report


def regressions(before, after):
    failures = []
    for name, current in after["files"].items():
        previous = before["files"].get(name, {"lengths": [], "complexity": 0, "wrappers": 0})
        if any(new > old for old, new in zip_longest(previous["lengths"], current["lengths"], fillvalue=0)):
            failures.append(f"{name}: lengths {previous['lengths']} -> {current['lengths']}")
        for key in ("complexity", "wrappers"):
            if current[key] > previous[key]:
                failures.append(f"{name}: {key} {previous[key]} -> {current[key]}")
    failures.extend(f"litter: {name}" for name in sorted(set(after["litter"]) - set(before["litter"])))
    for name, count in after["process_identifiers"].items():
        previous = before["process_identifiers"].get(name, 0)
        if count > previous:
            failures.append(f"{name}: process identifiers {previous} -> {count}")
    return failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--measure", action="store_true", help="print measurements for an explicit baseline review")
    args = parser.parse_args()
    current = measure(args.root)
    if args.measure:
        print(json.dumps(current, indent=2, sort_keys=True))
        return 0
    document = args.root / "docs/quality.md"
    text = document.read_text()
    if text.count(BASELINE) != 1:
        raise ValueError(f"{document}: expected exactly one quality baseline")
    baseline = json.loads(text.split(BASELINE)[1].split("\n```", 1)[0])
    failures = regressions(baseline, current)
    if failures:
        print("quality baseline regressed:\n" + "\n".join(failures), file=sys.stderr)
        return 1
    print(f"quality baseline passed ({len(current['files'])} source files)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
