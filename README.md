# Universum substrate experiment

This project does **not** translate Vulkan into smaller operations. It starts
with a material field and asks whether stable, useful organization can emerge
from local physical laws under a visible boundary condition.

The first experiment contains only:

- spectral energy;
- a material permeability that changes where energy repeatedly flows;
- local exchange between adjacent sites;
- continuous dissipation;
- local conversion of compatible energy into persistent visible radiation;
- external stimulation at one boundary.

There are deliberately no devices, API calls, handles, queues, command
buffers, memory allocations, shaders, fences, or schedules in the substrate.
Those concepts must not be planted in the experiment and renamed.

## Hypothesis

A field with local transport, reinforcement, erosion, spectral affinity, and
finite energy can form persistent pathways that transform boundary stimulation
into a stable visible field. The pathways are consequences of the laws; they
are not prescribed routes.

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
~~~

Identical dimensions, seed, laws, and stimulation produce identical results.

## What would falsify this stage

The experiment fails its first claim if any of these remain true after a run:

1. energy cannot be accounted for as resident, radiated, or dissipated;
2. repeated flow does not alter the material field;
3. no energy becomes visible away from the stimulation boundary;
4. the resulting organization is entirely fixed by initialization rather than
   being affected by ongoing interaction;
5. identical initial conditions do not yield identical measured evolution.

Passing these conditions only earns the right to build the next experiment.
It does not establish equivalence to Vulkan or to a hardware driver.

## Current boundary condition

The fixed spectral coupling inside the field defines where energy may become
visible; it does not define a path from the stimulus to those locations. The
shape is intentionally just an experimental boundary. A later experiment can
replace it with matter that develops its own coupling through selection and
feedback.
