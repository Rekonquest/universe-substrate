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
- `phase-relay`: local phase-coherent spectral relay boosts adjacent
  conductance only when material and spectral shapes align.
- `phase-relay-transport`: phase relay compounded with moderate transport
  pressure.
- `phase-relay-low-leak`: phase relay compounded with lower dissipation and
  slower permeability erosion.
- `guarded-phase-relay`: phase relay with local source-band spectral guard.
- `guarded-phase-relay-low-leak`: guarded phase relay compounded with lower
  dissipation.
- `guarded-phase-relay-balanced`: guarded phase relay with moderate transport
  and moderate low leak.

## Salvage attempts

Each candidate includes a salvage attempt in
`artifacts/primitive-optimization-sweep.txt`. The important point is that the
sweep tests softened or paired versions of aggressive primitives instead of
declaring a primitive failed after one naive stack.

## Release evidence

Source report: `artifacts/primitive-optimization-sweep.txt`

- sweep gate: `PASS`
- all deterministic: `true`
- max relative accounting error: `0.000000094309`
- best channel information: `baseline-adaptive`
- best channel-information bits: `0.571924141`
- best per-moment signal rate: `phase-relay-low-leak`
- best per-moment signal-rate value: `0.250970884`
- best radiation rate: `low-leak-plus-radiation`
- best per-moment radiation-rate value: `0.804659432`

Signal-rate and radiation-rate values are deterministic per substrate moment.
Wall-clock elapsed time is still reported separately, but it is not used for
primitive leader selection.

Guarded relay evidence:

- `guarded-phase-relay` reduced the single-seed channel-information loss from
  the unguarded relay's `-0.009203312` bits to `-0.004773906` bits while still
  increasing channel signal by `1.088452961x`.
- `guarded-phase-relay-low-leak` kept most of the low-leak signal gain with
  `1.240432901x` channel signal, but still did not beat baseline channel
  information in the single-seed run.

## Operator primitive impact

operator_primitive_impact: The sweep found speed-adjacent gains, but the
default adaptive stack still preserved the highest channel information in this
run. `phase-relay-low-leak` improved signal rate in the current single-seed
sweep, and `low-leak-plus-radiation` improved radiation rate. These are
candidates for follow-up, not approved replacements. `guarded-phase-relay`
is a salvage candidate for reducing information loss, not a replacement.

operator_notification: No primitive was rejected. Any future candidate
disposition must include the exact primitive list, stack order, measured
outcomes, deterministic repeat hash, and salvage attempt before operator
approval.
