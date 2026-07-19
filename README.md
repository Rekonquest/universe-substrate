# Universum substrate experiment

This project does **not** translate Vulkan into smaller operations. It starts
with a material field and asks whether stable, useful organization can emerge
from local physical laws under an observation boundary.

The current experiment contains only:

- spectral energy;
- a material permeability that changes where energy repeatedly flows;
- deterministic material phase that changes local edge conductance;
- local exchange between adjacent sites;
- continuous dissipation;
- weak spectral receptivity that compounds where repeated compatible flow
  succeeds;
- local conversion of compatible energy into persistent visible radiation;
- optional disturbance modes that either scar the material or inject tracked
  noise;
- observation-side channel measurements that estimate how much red, green, and
  blue source structure survives transport;
- external stimulation at one boundary.

There are deliberately no devices, API calls, handles, queues, command
buffers, memory allocations, shaders, fences, or schedules in the substrate.
Those concepts must not be planted in the experiment and renamed.

## Hypothesis

A field with local transport, reinforcement, erosion, compounding spectral
coupling, and finite energy can form persistent pathways that transform
boundary stimulation into a stable visible field. The pathways are
consequences of the laws; they are not prescribed routes.

This does not yet claim to be a GPU driver. It is the first executable test of
the more basic claim that an outcome-producing computational structure can
organize before a human driver ontology is introduced.

## Run

~~~powershell
cargo run --release
~~~

The run produces universe-frame.bmp for the visible state and
universe-frame.txt for its energy and organization measurements under the
artifacts directory.

Useful controls:

~~~powershell
cargo run --release -- --width 256 --height 144 --moments 4000 --seed 7
cargo run --release -- --coupling fixed --output artifacts/stage1-fixed.bmp
cargo run --release -- --coupling inert --output artifacts/no-learning.bmp
cargo run --release -- --disturbance scar --output artifacts/scar-adaptive.bmp
cargo run --release -- --coupling inert --disturbance scar --output artifacts/scar-inert.bmp
cargo run --release -- --disturbance noise --output artifacts/noisy-adaptive.bmp
cargo run --release -- --disturbance scar --sample-every 250 --output artifacts/scar-timeline.bmp
cargo run --release --bin falsify -- --output artifacts/primitive-stack-falsification.txt
cargo run --release --bin sweep -- --output artifacts/primitive-optimization-sweep.txt
~~~

The `adaptive` mode is the default. It starts with tiny noisy coupling and lets
repeated successful flow compound that coupling. The `fixed` mode preserves the
Stage 1 authored aperture for comparison. The `inert` mode keeps the weak seed
coupling but disables learning.

The `none` disturbance mode is the default. The `scar` disturbance absorbs
energy and weakens material along a deterministic injury in the field. The
`noise` disturbance injects sparse spectral perturbation energy and tracks that
energy separately from boundary stimulation.

Reports include energy accounting, organization, disturbance accounting, and
channel measurements. `channel_signal` is the coherent diagonal of the
source-band/color matrix. `channel_fidelity` measures diagonal signal against
crosstalk. `channel_balance` falls when one source band dominates the others.
`channel_information_bits` is a mutual-information style estimate over the
observed source-band/color matrix.

When `--sample-every N` is provided, the run also writes a CSV timeline beside
the bitmap and text report. The timeline records the full measurement set at
age zero, each N-moment interval, and the final age. This is useful for seeing
whether organization compounds, saturates, collapses, or recovers under a
disturbance.

Identical dimensions, seed, coupling mode, laws, and stimulation produce
identical results.

## Primitive-stack falsification gate

`universum-falsify` runs the current substrate through the same ordered
primitive stack under several controlled boundaries before any primitive can be
called an enhancement or discarded:

1. boundary stimulation;
2. local energy flow;
3. permeability formation;
4. erosion;
5. spectral coupling;
6. radiation;
7. dissipation.

The gate currently compares adaptive, inert, fixed, scarred, and noisy stacks.
It requires deterministic repeated hashes, bounded energy-accounting error,
adaptive radiation gain over inert material, adaptive channel-information gain
over inert material, adaptive scar survival over inert scar, and tracked noise
energy under the noise boundary.

The report intentionally records `operator_disposition=none`. A failed stack
does not remove or discard a primitive by itself. It only produces evidence for
operator review.

The current release run wrote
`artifacts/primitive-stack-falsification.txt` and passed with:

- adaptive radiation gain: 39.953245299x;
- adaptive channel-information gain: 0.213472535 bits;
- scar radiation gain: 47.955535563x;
- scar channel-information gain: 0.283003310 bits;
- deterministic visible hashes for all six tested stacks.

## Primitive optimization sweep

`universum-sweep` tests candidate primitive compounds against the default
adaptive stack. The current sweep keeps the same stack order and changes only
law strengths such as transport pressure, coupling formation, dissipation,
erosion, and radiation conversion. It records every tested primitive before any
summary line and always writes `operator_rejection_count=0`.

The current release sweep wrote `artifacts/primitive-optimization-sweep.txt`
and tested:

- `transport-pressure`;
- `coupling-compound`;
- `low-leak-memory`;
- `radiation-gate`;
- `transport-plus-coupling`;
- `low-leak-plus-radiation`;
- `balanced-channel-compound`.

The current result is mixed and therefore stays under operator review:

- best channel information: `baseline-adaptive` at `0.571924141` bits;
- best signal rate: `transport-pressure`;
- best radiation rate: `low-leak-plus-radiation`;
- all candidate repeated hashes were deterministic;
- max relative accounting error was `0.000000080395`;
- no primitive was rejected.

## What would falsify this stage

The experiment fails its first claim if any of these remain true after a run:

1. energy cannot be accounted for as resident, radiated, or dissipated;
2. repeated flow does not alter the material field;
3. no energy becomes visible away from the stimulation boundary;
4. the resulting organization is entirely fixed by initialization rather than
   being affected by ongoing interaction;
5. identical initial conditions do not yield identical measured evolution.
6. adaptive coupling does not measurably outperform inert coupling under the
   same seed and boundary stimulation.
7. under a scar disturbance, adaptive material cannot preserve substantially
   more visible radiation than inert material under the same injury.
8. disturbance energy is not accounted for as introduced or dissipated.
9. adaptive material cannot preserve substantially more channel signal and
   channel information than inert material under the same scar.
10. time-sampled runs show no measurable formation trajectory, only a static
    final-state artifact.
11. the primitive-stack falsification gate does not pass determinism,
    accounting, adaptive gain, scar resilience, and noise-accounting checks.
12. optimization sweeps claim a primitive improvement without reporting the
    tested primitive list, stack order, deterministic repeat hash, accounting
    error, salvage attempt, and operator disposition.

Passing these conditions only earns the right to build the next experiment.
It does not establish equivalence to Vulkan or to a hardware driver.

## Current boundary condition

The field still has an observation-side access gradient so energy is not
rendered equally at the stimulation edge. In the default `adaptive` mode, that
gradient does not prescribe a visible shape. Coupling starts weak and noisy,
then compounds only where local transport and compatible signal repeatedly
support it.
