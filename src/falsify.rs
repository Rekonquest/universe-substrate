use std::{fmt, time::Instant};

use crate::{Config, CouplingMode, DisturbanceMode, Measurements, World};

const STANDARD_ORDER: &str = "boundary_stimulation -> local_energy_flow -> permeability_formation -> erosion -> spectral_coupling -> radiation -> dissipation";

#[derive(Clone, Copy, Debug)]
pub struct PrimitiveStack {
    pub name: &'static str,
    pub order: &'static str,
    pub coupling_mode: CouplingMode,
    pub disturbance_mode: DisturbanceMode,
    pub width: usize,
    pub height: usize,
    pub seed: u64,
    pub moments: u64,
}

impl PrimitiveStack {
    pub const fn standard(
        name: &'static str,
        coupling_mode: CouplingMode,
        disturbance_mode: DisturbanceMode,
        width: usize,
        height: usize,
        seed: u64,
        moments: u64,
    ) -> Self {
        Self {
            name,
            order: STANDARD_ORDER,
            coupling_mode,
            disturbance_mode,
            width,
            height,
            seed,
            moments,
        }
    }

    fn config(self) -> Config {
        Config {
            width: self.width,
            height: self.height,
            seed: self.seed,
            coupling_mode: self.coupling_mode,
            disturbance_mode: self.disturbance_mode,
            ..Config::default()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StackOutcome {
    pub stack: PrimitiveStack,
    pub measurements: Measurements,
    pub elapsed_seconds: f64,
    pub visible_hash: u64,
    pub repeated_visible_hash: u64,
    pub deterministic: bool,
    pub relative_accounting_error: f64,
}

impl StackOutcome {
    pub fn evolve(stack: PrimitiveStack) -> Result<Self, String> {
        let started = Instant::now();
        let mut world = World::new(stack.config())?;
        world.evolve_for(stack.moments);
        let elapsed_seconds = started.elapsed().as_secs_f64();
        let measurements = world.measurements();
        let visible_hash = hash_visible(&world);

        let mut repeated = World::new(stack.config())?;
        repeated.evolve_for(stack.moments);
        let repeated_visible_hash = hash_visible(&repeated);
        let repeated_measurements = repeated.measurements();
        let deterministic = visible_hash == repeated_visible_hash
            && same_measurements(measurements, repeated_measurements);
        let relative_accounting_error = if measurements.introduced.abs() <= f64::EPSILON {
            measurements.accounting_error.abs()
        } else {
            measurements.accounting_error.abs() / measurements.introduced.abs()
        };

        Ok(Self {
            stack,
            measurements,
            elapsed_seconds,
            visible_hash,
            repeated_visible_hash,
            deterministic,
            relative_accounting_error,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FalsificationThresholds {
    pub max_relative_accounting_error: f64,
    pub min_adaptive_radiation_gain: f64,
    pub min_adaptive_channel_info_gain: f64,
    pub min_scar_radiation_gain: f64,
    pub min_scar_channel_info_gain: f64,
    pub min_noise_luminous_sites: usize,
}

impl Default for FalsificationThresholds {
    fn default() -> Self {
        Self {
            max_relative_accounting_error: 0.000_05,
            min_adaptive_radiation_gain: 8.0,
            min_adaptive_channel_info_gain: 0.08,
            min_scar_radiation_gain: 4.0,
            min_scar_channel_info_gain: 0.15,
            min_noise_luminous_sites: 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FalsificationReport {
    pub thresholds: FalsificationThresholds,
    pub baseline: StackOutcome,
    pub inert: StackOutcome,
    pub fixed: StackOutcome,
    pub scar_adaptive: StackOutcome,
    pub scar_inert: StackOutcome,
    pub noise_adaptive: StackOutcome,
}

impl FalsificationReport {
    pub fn passed(&self) -> bool {
        self.failures().is_empty()
    }

    pub fn failures(&self) -> Vec<&'static str> {
        let mut failures = Vec::new();
        for outcome in self.outcomes() {
            if !outcome.deterministic {
                failures.push("identical stack run produced different visible or measured state");
            }
            if outcome.relative_accounting_error > self.thresholds.max_relative_accounting_error {
                failures.push("energy accounting error exceeded threshold");
            }
        }
        if self.baseline.measurements.radiated
            <= self.inert.measurements.radiated * self.thresholds.min_adaptive_radiation_gain
        {
            failures.push("adaptive stack did not radiate enough beyond inert coupling");
        }
        if self.baseline.measurements.channel_information_bits
            <= self.inert.measurements.channel_information_bits
                + self.thresholds.min_adaptive_channel_info_gain
        {
            failures.push(
                "adaptive stack did not add enough channel information beyond inert coupling",
            );
        }
        if self.scar_adaptive.measurements.radiated
            <= self.scar_inert.measurements.radiated * self.thresholds.min_scar_radiation_gain
        {
            failures
                .push("adaptive scar stack did not preserve enough radiation beyond inert scar");
        }
        if self.scar_adaptive.measurements.channel_information_bits
            <= self.scar_inert.measurements.channel_information_bits
                + self.thresholds.min_scar_channel_info_gain
        {
            failures.push("adaptive scar stack did not preserve enough channel information");
        }
        if self.noise_adaptive.measurements.disturbance_introduced <= 0.0 {
            failures.push("noise stack did not introduce tracked disturbance energy");
        }
        if self.noise_adaptive.measurements.luminous_sites
            < self.thresholds.min_noise_luminous_sites
        {
            failures.push("noise stack produced no visible state away from accounting noise");
        }
        failures
    }

    pub fn outcomes(&self) -> [StackOutcome; 6] {
        [
            self.baseline,
            self.inert,
            self.fixed,
            self.scar_adaptive,
            self.scar_inert,
            self.noise_adaptive,
        ]
    }
}

pub fn run_standard_falsification(
    width: usize,
    height: usize,
    seed: u64,
    moments: u64,
    thresholds: FalsificationThresholds,
) -> Result<FalsificationReport, String> {
    Ok(FalsificationReport {
        thresholds,
        baseline: StackOutcome::evolve(PrimitiveStack::standard(
            "adaptive-none",
            CouplingMode::Adaptive,
            DisturbanceMode::None,
            width,
            height,
            seed,
            moments,
        ))?,
        inert: StackOutcome::evolve(PrimitiveStack::standard(
            "inert-none",
            CouplingMode::Inert,
            DisturbanceMode::None,
            width,
            height,
            seed,
            moments,
        ))?,
        fixed: StackOutcome::evolve(PrimitiveStack::standard(
            "fixed-none",
            CouplingMode::Fixed,
            DisturbanceMode::None,
            width,
            height,
            seed,
            moments,
        ))?,
        scar_adaptive: StackOutcome::evolve(PrimitiveStack::standard(
            "adaptive-scar",
            CouplingMode::Adaptive,
            DisturbanceMode::Scar,
            width,
            height,
            seed,
            moments,
        ))?,
        scar_inert: StackOutcome::evolve(PrimitiveStack::standard(
            "inert-scar",
            CouplingMode::Inert,
            DisturbanceMode::Scar,
            width,
            height,
            seed,
            moments,
        ))?,
        noise_adaptive: StackOutcome::evolve(PrimitiveStack::standard(
            "adaptive-noise",
            CouplingMode::Adaptive,
            DisturbanceMode::Noise,
            width,
            height,
            seed,
            moments,
        ))?,
    })
}

impl fmt::Display for FalsificationReport {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(output, "Universum primitive-stack falsification report")?;
        writeln!(output, "schema=1")?;
        writeln!(output, "operator_disposition=none")?;
        writeln!(output, "stacking_order_critical=true")?;
        writeln!(output, "all_primitives_reported_before_disposition=true")?;
        writeln!(
            output,
            "max_relative_accounting_error={:.9}",
            self.thresholds.max_relative_accounting_error
        )?;
        writeln!(output)?;
        for outcome in self.outcomes() {
            write_outcome(output, outcome)?;
        }
        writeln!(
            output,
            "adaptive_radiation_gain={:.9}",
            ratio(
                self.baseline.measurements.radiated,
                self.inert.measurements.radiated
            )
        )?;
        writeln!(
            output,
            "adaptive_channel_information_gain={:.9}",
            self.baseline.measurements.channel_information_bits
                - self.inert.measurements.channel_information_bits
        )?;
        writeln!(
            output,
            "scar_radiation_gain={:.9}",
            ratio(
                self.scar_adaptive.measurements.radiated,
                self.scar_inert.measurements.radiated
            )
        )?;
        writeln!(
            output,
            "scar_channel_information_gain={:.9}",
            self.scar_adaptive.measurements.channel_information_bits
                - self.scar_inert.measurements.channel_information_bits
        )?;
        if self.passed() {
            writeln!(output, "falsification_gate=PASS")?;
        } else {
            writeln!(output, "falsification_gate=FAIL")?;
            for failure in self.failures() {
                writeln!(output, "failure={failure}")?;
            }
        }
        Ok(())
    }
}

fn write_outcome(output: &mut fmt::Formatter<'_>, outcome: StackOutcome) -> fmt::Result {
    writeln!(output, "stack={}", outcome.stack.name)?;
    writeln!(output, "  order={}", outcome.stack.order)?;
    writeln!(
        output,
        "  coupling={}",
        outcome.stack.coupling_mode.as_str()
    )?;
    writeln!(
        output,
        "  disturbance={}",
        outcome.stack.disturbance_mode.as_str()
    )?;
    writeln!(output, "  width={}", outcome.stack.width)?;
    writeln!(output, "  height={}", outcome.stack.height)?;
    writeln!(output, "  seed={}", outcome.stack.seed)?;
    writeln!(output, "  moments={}", outcome.stack.moments)?;
    writeln!(output, "  elapsed_seconds={:.6}", outcome.elapsed_seconds)?;
    writeln!(
        output,
        "  introduced={:.9}",
        outcome.measurements.introduced
    )?;
    writeln!(output, "  resident={:.9}", outcome.measurements.resident)?;
    writeln!(output, "  radiated={:.9}", outcome.measurements.radiated)?;
    writeln!(
        output,
        "  dissipated={:.9}",
        outcome.measurements.dissipated
    )?;
    writeln!(
        output,
        "  disturbance_introduced={:.9}",
        outcome.measurements.disturbance_introduced
    )?;
    writeln!(
        output,
        "  relative_accounting_error={:.12}",
        outcome.relative_accounting_error
    )?;
    writeln!(
        output,
        "  organized_fraction={:.9}",
        outcome.measurements.organized_fraction
    )?;
    writeln!(
        output,
        "  coupled_fraction={:.9}",
        outcome.measurements.coupled_fraction
    )?;
    writeln!(
        output,
        "  largest_organized_component={:.9}",
        outcome.measurements.largest_organized_component
    )?;
    writeln!(
        output,
        "  luminous_sites={}",
        outcome.measurements.luminous_sites
    )?;
    writeln!(
        output,
        "  channel_fidelity={:.9}",
        outcome.measurements.channel_fidelity
    )?;
    writeln!(
        output,
        "  channel_balance={:.9}",
        outcome.measurements.channel_balance
    )?;
    writeln!(
        output,
        "  channel_information_bits={:.9}",
        outcome.measurements.channel_information_bits
    )?;
    writeln!(output, "  visible_hash=0x{:016x}", outcome.visible_hash)?;
    writeln!(
        output,
        "  repeated_visible_hash=0x{:016x}",
        outcome.repeated_visible_hash
    )?;
    writeln!(output, "  deterministic={}", outcome.deterministic)?;
    Ok(())
}

fn same_measurements(a: Measurements, b: Measurements) -> bool {
    a.age == b.age
        && a.introduced.to_bits() == b.introduced.to_bits()
        && a.resident.to_bits() == b.resident.to_bits()
        && a.radiated.to_bits() == b.radiated.to_bits()
        && a.dissipated.to_bits() == b.dissipated.to_bits()
        && a.accounting_error.to_bits() == b.accounting_error.to_bits()
        && a.disturbance_introduced.to_bits() == b.disturbance_introduced.to_bits()
        && a.disturbance_dissipated.to_bits() == b.disturbance_dissipated.to_bits()
        && a.mean_permeability.to_bits() == b.mean_permeability.to_bits()
        && a.mean_coupling.to_bits() == b.mean_coupling.to_bits()
        && a.organized_fraction.to_bits() == b.organized_fraction.to_bits()
        && a.coupled_fraction.to_bits() == b.coupled_fraction.to_bits()
        && a.largest_organized_component.to_bits() == b.largest_organized_component.to_bits()
        && a.luminous_sites == b.luminous_sites
        && a.channel_signal.to_bits() == b.channel_signal.to_bits()
        && a.channel_total.to_bits() == b.channel_total.to_bits()
        && a.channel_fidelity.to_bits() == b.channel_fidelity.to_bits()
        && a.channel_balance.to_bits() == b.channel_balance.to_bits()
        && a.channel_information_bits.to_bits() == b.channel_information_bits.to_bits()
}

fn hash_visible(world: &World) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for pixel in world.rgb8() {
        for channel in pixel {
            hash ^= u64::from(channel);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

fn ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() <= f64::EPSILON {
        f64::INFINITY
    } else {
        numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_harness_preserves_determinism_and_accounting() {
        let report =
            run_standard_falsification(40, 28, 103, 650, FalsificationThresholds::default())
                .unwrap();
        assert!(report.passed(), "{}", report);
    }

    #[test]
    fn report_names_every_stack_before_any_disposition() {
        let report =
            run_standard_falsification(32, 24, 103, 420, FalsificationThresholds::default())
                .unwrap();
        let text = report.to_string();
        let disposition = text.find("falsification_gate=").unwrap();
        for stack in [
            "stack=adaptive-none",
            "stack=inert-none",
            "stack=fixed-none",
            "stack=adaptive-scar",
            "stack=inert-scar",
            "stack=adaptive-noise",
        ] {
            let position = text
                .find(stack)
                .unwrap_or_else(|| panic!("missing {stack}"));
            assert!(position < disposition, "{stack} was not reported first");
        }
        assert!(text.contains("operator_disposition=none"));
        assert!(text.contains("stacking_order_critical=true"));
    }
}
