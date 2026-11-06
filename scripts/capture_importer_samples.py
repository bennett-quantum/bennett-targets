#!/usr/bin/env python3
"""Refresh real external fixtures; requires qualtran==0.7.0 and network access.

This is an explicit maintenance command, never part of the offline test run.
It runs Qualtran's resource counter and retrieves Microsoft's estimator report.
"""
import hashlib
from importlib.metadata import version
import json
from pathlib import Path
import subprocess
from urllib.request import urlopen


def main():
    from qualtran.bloqs.qft import QFTTextBook
    from qualtran.resource_counting import QECGatesCost, get_cost_value

    if version("qualtran") != "0.7.0":
        raise RuntimeError("capture requires qualtran==0.7.0")
    destination = Path(__file__).resolve().parents[1] / "importers/fixtures"
    retrieved = subprocess.check_output(["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], text=True).strip()
    qualtran = (json.dumps(get_cost_value(QFTTextBook(8), QECGatesCost()).asdict(), indent=2) + "\n").encode()
    sha = "e5d6dbeabc5b4fcb557269c7aa37a03c79ae4e85"
    url = f"https://raw.githubusercontent.com/microsoft/qdk/{sha}/source/resource_estimator/src/system/test_report.json"
    with urlopen(url, timeout=30) as response:
        azure = response.read()
    report = json.loads(azure)
    projection = (json.dumps({"qubitParams": report["jobParams"]["qubitParams"]}, indent=2, ensure_ascii=False) + "\n").encode()
    sources = {
        "qualtran-qft8-real.json": (qualtran, {
            "source_url": "https://pypi.org/project/qualtran/0.7.0/",
            "producer": "qualtran==0.7.0",
            "recipe": "get_cost_value(QFTTextBook(8), QECGatesCost()).asdict()",
            "scope": "Actual resource-counter output, not measured hardware; asdict omits zero fields.",
        }),
        "azure-estimator-report-real.json": (azure, {
            "source_url": url,
            "commit": sha,
            "scope": "Unmodified upstream resource-estimator test_report.json output artifact; not a cloud submission or measured hardware.",
        }),
        "azure-estimator-qubit-params-real.json": (projection, {
            "source_url": url,
            "commit": sha,
            "extraction": "/jobParams/qubitParams, wrapped in {qubitParams: ...}; every value unchanged",
            "scope": "Hardware-parameter projection only; the full QEC/physical report is unsupported by this adapter and tested as a rejection.",
        }),
    }
    for name, (data, metadata) in sources.items():
        (destination / name).write_bytes(data)
        metadata.update(retrieved_utc=retrieved, sha256=hashlib.sha256(data).hexdigest())
        (destination / name.replace(".json", ".provenance.json")).write_text(json.dumps(metadata, indent=2) + "\n")


if __name__ == "__main__":
    main()
