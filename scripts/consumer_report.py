"""Stable contract diagnostics for consumer reports, independent of issue tracking."""
import re


def stable_report(value):
    """Keep upstream diagnostics and contract citations, without tracking IDs."""
    if isinstance(value, str):
        return re.sub(r"\bb-(?:isa-spec|targets|vm)-[a-z0-9]+(?:-[a-z0-9]+)*\b", "tracked contract issue", value)
    if isinstance(value, list):
        return [stable_report(item) for item in value]
    if isinstance(value, dict):
        return {key: stable_report(item) for key, item in value.items()}
    return value
