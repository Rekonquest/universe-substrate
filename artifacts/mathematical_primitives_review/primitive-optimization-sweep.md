# Primitive optimization sweep review

schema=1

operator_disposition=none

operator_rejection_count=0

stacking_order_critical=true

## Purpose

This review records the first live optimization sweep for candidate primitive
compounds. The sweep tests data-speed-adjacent changes in local transport,
spectral coupling, dissipation, erosion, and radiation conversion. It does not
reject any primitive and does not rename a speed gain into a quality gain.

## Stack order preserved

1. boundary stimulation
2. local energy flow
3. permeability formation
4. erosion
5. spectral coupling
6. radiation
7. dissipation

## Candidate primitives tested

- `baseline-adaptive`: default reference stack.
- `transport-pressure`: higher local diffusion plus steeper boundary pressure.
- `coupling-compound`: faster spectral coupling formation plus stronger unused
  coupling erosion.
- `low-leak-memory`: lower dissipation plus slower permeability erosion.
- `radiation-gate`: higher radiation conversion plus lower coupling formation.
- `transport-plus-coupling`: transport-pressure compounded with faster coupling
  formation.
- `low-leak-plus-radiation`: lower dissipation compounded with higher visible
  extraction.
- `balanced-channel-compound`: moderate transport, coupling, and leak changes
  stacked together.

## Salvage attempts

Each candidate includes a salvage attempt in
`artifacts/primitive-optimization-sweep.txt`. The important point is that the
sweep tests softened or paired versions of aggressive primitives instead of
declaring a primitive failed after one naive stack.

## Release evidence

Source report: `artifacts/primitive-optimization-sweep.txt`

- sweep gate: `PASS`
- all deterministic: `true`
- max relative accounting error: `0.000000080395`
- best channel information: `baseline-adaptive`
- best channel-information bits: `0.571924141`
- best signal rate: `transport-pressure`
- best radiation rate: `low-leak-plus-radiation`

The exact signal-rate and radiation-rate values are left in the generated
report because they include wall-clock timing and can shift between otherwise
identical deterministic runs.

## Operator primitive impact

operator_primitive_impact: The sweep found speed-adjacent gains, but the
default adaptive stack still preserved the highest channel information in this
run. `transport-pressure` improved signal rate and
`low-leak-plus-radiation` improved radiation rate. These are candidates for
follow-up, not approved replacements.

operator_notification: No primitive was rejected. Any future candidate
disposition must include the exact primitive list, stack order, measured
outcomes, deterministic repeat hash, and salvage attempt before operator
approval.
