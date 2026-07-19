# Primitive-stack falsification review

schema=1

operator_disposition=none

stacking_order_critical=true

## Purpose

This review records the first automated falsification gate for the current
universe-native substrate. It does not reject, remove, or demote a primitive.
It reports the tested stacks, the stack order, the measured outcomes, and the
evidence that would be used for operator review.

## Stack order tested

1. boundary stimulation
2. local energy flow
3. permeability formation
4. erosion
5. spectral coupling
6. radiation
7. dissipation

## Primitive stacks tested

- `adaptive-none`: full adaptive stack with no disturbance.
- `inert-none`: same boundary and material field, but spectral coupling does
  not learn.
- `fixed-none`: authored fixed observation coupling for comparison.
- `adaptive-scar`: adaptive stack under deterministic material scarring.
- `inert-scar`: inert stack under the same deterministic material scarring.
- `adaptive-noise`: adaptive stack under tracked sparse spectral noise.

## Acceptance checks

- Every stack must repeat to the same visible hash and identical measurements.
- Relative energy-accounting error must stay below `0.00005`.
- Adaptive radiation must exceed inert radiation by at least `8x`.
- Adaptive channel information must exceed inert channel information by at
  least `0.08` bits.
- Adaptive scar radiation must exceed inert scar radiation by at least `4x`.
- Adaptive scar channel information must exceed inert scar channel information
  by at least `0.15` bits.
- Noise disturbance must introduce tracked disturbance energy and preserve a
  visible field.

## Release evidence

Source report: `artifacts/primitive-stack-falsification.txt`

- adaptive radiation gain: `39.953245299x`
- adaptive channel-information gain: `0.213472535` bits
- scar radiation gain: `47.955535563x`
- scar channel-information gain: `0.283003310` bits
- falsification gate: `PASS`

## Operator primitive impact

operator_primitive_impact: The current compounded adaptive stack has measured
support under the tested boundaries. No primitive has been removed or marked
for rejection. Any future failing primitive stack must be reported with its
specific primitive list, stack order, evidence, and attempted salvage path
before operator disposition.

operator_notification: All tested primitive stacks were reported before the
gate verdict. The report records `operator_disposition=none`.
