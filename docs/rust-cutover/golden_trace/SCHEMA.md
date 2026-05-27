# Golden Trace Schema

A golden trace is JSONL. Each line is one case.

Required fields:

```json
{
  "case_id": "execution.submit_accept_fill.001",
  "description": "Submit limit order and receive accepted/fill events",
  "input": {
    "events": []
  },
  "expected": {
    "events": []
  },
  "tolerances": {}
}
```

Rules:

- Use strings for decimal values.
- Use nanosecond timestamps as integers or strings.
- Do not rely on wall-clock time.
- Any nondeterministic field must be normalized or excluded with a documented tolerance.
