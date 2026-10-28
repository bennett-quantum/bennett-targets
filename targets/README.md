# targets

Each `*.json` file is a target description in RFC 8785 canonical form (sorted
keys, no whitespace, no trailing newline); `target_hash` is the SHA-256 of the
file bytes. Schema: bennett-isa-spec section 8.

`example-surface-code-d17.json` is an ILLUSTRATIVE example, with made-up round
numbers. It is not a real vendor model. Time units are seconds, here stylised
as code-cycle multiples; `qubit_second` rates are per-class and set to 1.
