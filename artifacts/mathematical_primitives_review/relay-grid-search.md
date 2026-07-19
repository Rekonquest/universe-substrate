# Relay/guard low-leak grid review

schema=1

operator_disposition=none

operator_rejection_count=0

stacking_order_critical=true

## Purpose

This review records the first bounded grid search over `phase_relay` and
`relay_guard` under the low-leak compound. The goal is to stop hand-picking
relay strengths and instead expose the speed/information/radiation tradeoff.

The grid runner lives in `tools/relay-grid` as a separate small crate so the
main substrate crate stays under the 4,000-line Rust crate limit.

## Stack order

1. boundary stimulation
2. local energy flow
3. phase relay
4. relay guard
5. permeability formation
6. erosion
7. spectral coupling
8. radiation
9. dissipation

## Grid

- phase relay values: `0.00`, `0.18`, `0.24`, `0.28`, `0.32`, `0.36`, `0.40`
- relay guard values: `0.00`, `0.25`, `0.50`, `0.75`, `1.00`
- low-leak compound: dissipation `0.00095`, erosion `0.00145`
- seeds: `4`
- candidate count: `35`

## Release evidence

Source report: `artifacts/relay-grid-search.txt`

- all deterministic: `true`
- max relative accounting error: `0.000000093053`
- Pareto frontier count: `5`
- grid gate: `PASS`

Pareto frontier:

- `relay-grid-p0.40-g0.00`
  - mean channel-information bits: `0.471795488`
  - mean signal rate: `0.289807896`
  - mean radiation rate: `0.860546847`
- `relay-grid-p0.40-g0.25`
  - mean channel-information bits: `0.474179629`
  - mean signal rate: `0.288273715`
  - mean radiation rate: `0.857359939`
- `relay-grid-p0.40-g0.50`
  - mean channel-information bits: `0.476600870`
  - mean signal rate: `0.286733011`
  - mean radiation rate: `0.854193277`
- `relay-grid-p0.40-g0.75`
  - mean channel-information bits: `0.479062093`
  - mean signal rate: `0.285184610`
  - mean radiation rate: `0.851030972`
- `relay-grid-p0.40-g1.00`
  - mean channel-information bits: `0.481557147`
  - mean signal rate: `0.283625295`
  - mean radiation rate: `0.847854626`

Adaptive baseline for this grid:

- mean channel-information bits: `0.496056783`
- mean signal rate: `0.234551001`
- mean radiation rate: `0.720250006`

## Operator primitive impact

operator_primitive_impact: The grid found a clear tradeoff frontier. Higher
guard preserves more channel information, while lower guard preserves more
signal and radiation rate. The best grid point for channel information still
does not beat adaptive baseline channel information, so this is tuning evidence
only.

operator_notification: No relay-grid candidate was rejected or promoted. Any
disposition still requires operator approval.
