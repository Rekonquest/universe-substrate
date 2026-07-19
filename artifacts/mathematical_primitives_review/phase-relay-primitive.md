# Phase relay primitive review

schema=1

operator_disposition=none

operator_rejection_count=0

stacking_order_critical=true

## Primitive

`phase_relay` is a local transport primitive. It boosts adjacent exchange only
when all of these are true:

- the two neighboring sites are phase-aligned;
- the material permeability bridge is already open;
- the spectral shapes on the two sides are compatible.

This is a data-speed-adjacent primitive, but it is still local energy flow. It
does not add queues, command buffers, non-local scheduling, handles, shaders,
or API objects.

## Stack order

1. boundary stimulation
2. local energy flow
3. phase/spectral relay inside local exchange
4. permeability formation
5. erosion
6. spectral coupling
7. radiation
8. dissipation

## Candidate stacks tested

- `phase-relay`
- `phase-relay-transport`
- `phase-relay-low-leak`
- `guarded-phase-relay`
- `guarded-phase-relay-low-leak`
- `guarded-phase-relay-balanced`

## Release evidence

Source reports:

- `artifacts/primitive-optimization-sweep.txt`
- `artifacts/primitive-multiseed-sweep.txt`
- `artifacts/phase-relay-smoke.txt`

Single-seed sweep:

- `phase-relay-low-leak` was best by signal rate.
- `phase-relay` increased radiation by `1.067700863x` but reduced channel
  information by `0.009203312` bits against baseline.
- `phase-relay-low-leak` increased radiation by `1.222611103x` and channel
  signal by `1.244729726x`, but reduced channel information by `0.033666287`
  bits against baseline.

Multi-seed sweep:

- `phase-relay-low-leak` won per-moment signal rate in `4` of `4` seeds.
- `guarded-phase-relay` won channel information in `2` of `4` seeds.
- `phase-relay` had `0` channel-information wins, `0` signal-rate wins, and
  `0` radiation-rate wins.
- `phase-relay-transport` had `0` wins in the tracked leader categories.
- all phase-relay candidate repeats were deterministic.

Smoke artifact:

- phase relay strength: `0.320000`
- luminous sites: `2910`
- channel-information bits: `1.261688882`
- bitmap SHA-256: `3E539AFA5706CEA89FCADB83CC77727C70F3938F0969950E4E65843371505C0D`

Guarded smoke artifact:

- phase relay strength: `0.300000`
- relay guard strength: `0.500000`
- luminous sites: `2887`
- channel-information bits: `1.265713355`
- bitmap SHA-256: `893FD9CF68820259E28BE79065B23F48CFBA53FF6C4C215E60A5ADD3DA670D8A`

## Operator primitive impact

operator_primitive_impact: `phase_relay` is now implemented and measured. It
is useful as a deterministic per-moment speed/throughput candidate when
compounded with low leak in this four-seed gate, but it has not beaten
baseline channel information across the tested seeds. It remains a candidate
for follow-up tuning, not a promoted replacement.

operator_primitive_impact: `relay_guard` is a useful local salvage primitive
for channel preservation. It improved the channel-information standing of the
relay family, but it did not displace the stable signal-rate or radiation-rate
leaders.

operator_notification: No phase-relay primitive was rejected. No phase-relay
primitive was promoted. Any disposition still requires operator approval.
