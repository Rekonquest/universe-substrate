# Multi-seed primitive optimization sweep review

schema=1

operator_disposition=none

operator_rejection_count=0

stacking_order_critical=true

## Purpose

This review records the first multi-seed primitive sweep. Its job is to reduce
false positives from one favorable seed. It repeats the same candidate stacks
under four deterministic seeds and aggregates leader counts, mean metrics,
determinism, and accounting error.

## Stack order preserved

1. boundary stimulation
2. local energy flow
3. permeability formation
4. erosion
5. spectral coupling
6. radiation
7. dissipation

## Candidate primitives tested

The same candidate set from `primitive-optimization-sweep.md` was tested:

- `baseline-adaptive`
- `transport-pressure`
- `coupling-compound`
- `low-leak-memory`
- `radiation-gate`
- `transport-plus-coupling`
- `low-leak-plus-radiation`
- `balanced-channel-compound`

## Release evidence

Source report: `artifacts/primitive-multiseed-sweep.txt`

- seed count: `4`
- all deterministic: `true`
- max relative accounting error: `0.000000090349`
- stable channel-information leader: `mixed`
- stable signal-rate leader: `mixed`
- stable radiation-rate leader: `mixed`
- multiseed sweep gate: `PASS`

Leader counts:

- `baseline-adaptive`: channel wins `3`, signal-rate wins `0`,
  radiation-rate wins `0`.
- `radiation-gate`: channel wins `1`, signal-rate wins `0`,
  radiation-rate wins `0`.
- `transport-pressure`: channel wins `0`, signal-rate wins `3`,
  radiation-rate wins `0`.
- `low-leak-memory`: channel wins `0`, signal-rate wins `1`,
  radiation-rate wins `1`.
- `low-leak-plus-radiation`: channel wins `0`, signal-rate wins `0`,
  radiation-rate wins `3`.

## Operator primitive impact

operator_primitive_impact: The single-seed speed result did not become a
universal promotion after multi-seed testing. `transport-pressure` remains a
strong signal-rate candidate, and `low-leak-plus-radiation` remains a strong
radiation-rate candidate, but neither preserves channel information better than
baseline across the tested seeds.

operator_notification: No primitive was rejected. Multi-seed evidence now
blocks promoting or rejecting a primitive from one seed alone.
