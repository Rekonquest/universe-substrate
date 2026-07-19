# Guarded phase relay primitive review

schema=1

operator_disposition=none

operator_rejection_count=0

stacking_order_critical=true

## Primitive

`relay_guard` is a local governor on `phase_relay`. It suppresses relay
amplification when the local spectrum drifts away from the expected source-band
shape at that vertical position. This keeps the relay local and physical:
there is no scheduler, queue, command buffer, handle, shader, or API object.

## Stack order

1. boundary stimulation
2. local energy flow
3. phase/spectral relay inside local exchange
4. source-band relay guard inside local exchange
5. permeability formation
6. erosion
7. spectral coupling
8. radiation
9. dissipation

## Candidate stacks tested

- `guarded-phase-relay`
- `guarded-phase-relay-low-leak`
- `guarded-phase-relay-balanced`

## Release evidence

Source reports:

- `artifacts/primitive-optimization-sweep.txt`
- `artifacts/primitive-multiseed-sweep.txt`
- `artifacts/guarded-phase-relay-smoke.txt`

Single-seed sweep:

- `guarded-phase-relay` channel-information gain: `-0.004773906` bits.
- `guarded-phase-relay` channel-signal gain: `1.088452961x`.
- `guarded-phase-relay-low-leak` channel-signal gain: `1.240432901x`.
- `guarded-phase-relay-low-leak` channel-information gain: `-0.030351598`
  bits.

Multi-seed sweep:

- `guarded-phase-relay` channel wins: `2` of `4`.
- `guarded-phase-relay-low-leak` signal-rate wins: `0` of `4`.
- `guarded-phase-relay-balanced` tracked leader wins: `0`.
- all guarded-relay candidate repeats were deterministic.

Smoke artifact:

- phase relay strength: `0.300000`
- relay guard strength: `0.500000`
- luminous sites: `2887`
- channel-information bits: `1.265713355`
- bitmap SHA-256: `893FD9CF68820259E28BE79065B23F48CFBA53FF6C4C215E60A5ADD3DA670D8A`

## Operator primitive impact

operator_primitive_impact: `relay_guard` is a promising channel-preservation
salvage primitive for the phase-relay family. It did not become the stable
signal-rate or radiation-rate leader, but it won channel information in half
the tested seeds. It remains a candidate for follow-up tuning.

operator_notification: No guarded-relay primitive was rejected or promoted.
Any disposition still requires operator approval.
