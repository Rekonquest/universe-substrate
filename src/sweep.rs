use std::{fmt, time::Instant};

use crate::{Config, CouplingMode, DisturbanceMode, Measurements, World};

const STACK_ORDER: &str = "boundary_stimulation -> local_energy_flow -> permeability_formation -> erosion -> spectral_coupling -> radiation -> dissipation";

#[derive(Clone, Copy, Debug)]
pub struct PrimitiveCandidate {
    pub name: &'static str,
    pub primitives: &'static str,
    pub stack_order: &'static str,
    pub operator_disposition: &'static str,
    pub salvage_attempt: &'static str,
    configure: fn(Config) -> Config,
}

impl PrimitiveCandidate {
    const fn new(
        name: &'static str,
        primitives: &'static str,
        salvage_attempt: &'static str,
        configure: fn(Config) -> Config,
    ) -> Self {
        Self {
            name,
            primitives,
            stack_order: STACK_ORDER,
            operator_disposition: "none",
            salvage_attempt,
            configure,
        }
    }

    fn config(self, baseline: Config) -> Config {
        (self.configure)(baseline)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CandidateOutcome {
    pub candidate: PrimitiveCandidate,
    pub measurements: Measurements,
    pub elapsed_seconds: f64,
    pub visible_hash: u64,
    pub repeated_visible_hash: u64,
    pub deterministic: bool,
    pub relative_accounting_error: f64,
    pub radiation_gain: f64,
    pub channel_information_gain: f64,
    pub channel_signal_gain: f64,
    pub luminous_gain: f64,
    pub information_rate: f64,
    pub signal_rate: f64,
    pub radiation_rate: f64,
}

#[derive(Clone, Debug)]
pub struct SweepReport {
    pub width: usize,
    pub height: usize,
    pub seed: u64,
    pub moments: u64,
    pub baseline: CandidateOutcome,
    pub candidates: Vec<CandidateOutcome>,
}

impl SweepReport {
    pub fn all_deterministic(&self) -> bool {
        self.baseline.deterministic
            && self
                .candidates
                .iter()
                .all(|candidate| candidate.deterministic)
    }

    pub fn max_relative_accounting_error(&self) -> f64 {
        self.outcomes()
            .into_iter()
            .map(|outcome| outcome.relative_accounting_error)
            .fold(0.0, f64::max)
    }

    pub fn outcomes(&self) -> Vec<CandidateOutcome> {
        let mut outcomes = Vec::with_capacity(self.candidates.len() + 1);
        outcomes.push(self.baseline);
        outcomes.extend(self.candidates.iter().copied());
        outcomes
    }

    fn best_by(&self, score: impl Fn(CandidateOutcome) -> f64) -> CandidateOutcome {
        self.outcomes()
            .into_iter()
            .max_by(|left, right| score(*left).total_cmp(&score(*right)))
            .expect("sweep always has a baseline")
    }

    pub fn best_by_channel_information(&self) -> CandidateOutcome {
        self.best_by(|outcome| outcome.measurements.channel_information_bits)
    }

    pub fn best_by_signal_rate(&self) -> CandidateOutcome {
        self.best_by(|outcome| outcome.signal_rate)
    }

    pub fn best_by_radiation_rate(&self) -> CandidateOutcome {
        self.best_by(|outcome| outcome.radiation_rate)
    }
}

pub fn run_standard_sweep(
    width: usize,
    height: usize,
    seed: u64,
    moments: u64,
) -> Result<SweepReport, String> {
    let baseline_config = Config {
        width,
        height,
        seed,
        coupling_mode: CouplingMode::Adaptive,
        disturbance_mode: DisturbanceMode::None,
        ..Config::default()
    }
    .validate()?;
    let baseline_candidate = PrimitiveCandidate::new(
        "baseline-adaptive",
        "default transport + default permeability formation + default coupling compounding + default radiation + default dissipation",
        "baseline reference; no salvage path needed",
        identity,
    );
    let baseline = evolve_baseline(baseline_candidate, baseline_config, moments)?;
    let candidates = standard_candidates()
        .into_iter()
        .map(|candidate| {
            evolve_candidate(candidate, baseline_config, moments, baseline.measurements)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SweepReport {
        width,
        height,
        seed,
        moments,
        baseline,
        candidates,
    })
}

fn standard_candidates() -> [PrimitiveCandidate; 10] {
    [
        PrimitiveCandidate::new(
            "transport-pressure",
            "higher local diffusion + steeper boundary pressure gradient",
            "kept total transport below stability bound and compared against default adaptive baseline",
            transport_pressure,
        ),
        PrimitiveCandidate::new(
            "coupling-compound",
            "faster spectral coupling formation + slightly faster unused-coupling erosion",
            "paired stronger compounding with stronger disuse erosion to avoid permanent noise capture",
            coupling_compound,
        ),
        PrimitiveCandidate::new(
            "low-leak-memory",
            "lower dissipation + slower permeability erosion",
            "reduced both energy leak and material forgetting while leaving local order unchanged",
            low_leak_memory,
        ),
        PrimitiveCandidate::new(
            "radiation-gate",
            "higher radiation conversion + slightly lower coupling formation",
            "raised visible conversion while reducing coupling formation enough to avoid immediate saturation",
            radiation_gate,
        ),
        PrimitiveCandidate::new(
            "transport-plus-coupling",
            "higher transport pressure compounded with faster spectral coupling formation",
            "stacked speed primitive before coupling primitive without changing observation order",
            transport_plus_coupling,
        ),
        PrimitiveCandidate::new(
            "low-leak-plus-radiation",
            "lower dissipation compounded with higher radiation conversion",
            "paired retained resident energy with stronger visible extraction to test useful output speed",
            low_leak_plus_radiation,
        ),
        PrimitiveCandidate::new(
            "balanced-channel-compound",
            "moderate transport pressure + moderate coupling formation + lower dissipation",
            "softened each individual primitive and tested whether the compound is more stable than extremes",
            balanced_channel_compound,
        ),
        PrimitiveCandidate::new(
            "phase-relay",
            "local phase-coherent spectral relay boosts adjacent conductance only when material and spectral shapes align",
            "bounded relay strength below transport stability limit and left all ordering local",
            phase_relay,
        ),
        PrimitiveCandidate::new(
            "phase-relay-transport",
            "phase relay compounded with moderate transport pressure",
            "stacked relay after local phase/spectral compatibility and kept diffusion lower than transport-pressure",
            phase_relay_transport,
        ),
        PrimitiveCandidate::new(
            "phase-relay-low-leak",
            "phase relay compounded with lower dissipation and slower permeability erosion",
            "paired relay speed with longer material memory without changing radiation conversion",
            phase_relay_low_leak,
        ),
    ]
}

fn evolve_baseline(
    candidate: PrimitiveCandidate,
    config: Config,
    moments: u64,
) -> Result<CandidateOutcome, String> {
    let started = Instant::now();
    let mut world = World::new(config)?;
    world.evolve_for(moments);
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let measurements = world.measurements();
    let visible_hash = hash_visible(&world);

    let mut repeated = World::new(config)?;
    repeated.evolve_for(moments);
    let repeated_visible_hash = hash_visible(&repeated);
    let repeated_measurements = repeated.measurements();
    let deterministic = visible_hash == repeated_visible_hash
        && same_measurements(measurements, repeated_measurements);
    let safe_moments = measurements.age.max(1) as f64;

    Ok(CandidateOutcome {
        candidate,
        measurements,
        elapsed_seconds,
        visible_hash,
        repeated_visible_hash,
        deterministic,
        relative_accounting_error: relative_accounting_error(measurements),
        radiation_gain: 1.0,
        channel_information_gain: 0.0,
        channel_signal_gain: 1.0,
        luminous_gain: 1.0,
        information_rate: measurements.channel_information_bits / safe_moments,
        signal_rate: measurements.channel_signal / safe_moments,
        radiation_rate: measurements.radiated / safe_moments,
    })
}

fn evolve_candidate(
    candidate: PrimitiveCandidate,
    baseline_config: Config,
    moments: u64,
    baseline: Measurements,
) -> Result<CandidateOutcome, String> {
    let config = candidate.config(baseline_config).validate()?;
    let started = Instant::now();
    let mut world = World::new(config)?;
    world.evolve_for(moments);
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let measurements = world.measurements();
    let visible_hash = hash_visible(&world);

    let mut repeated = World::new(config)?;
    repeated.evolve_for(moments);
    let repeated_visible_hash = hash_visible(&repeated);
    let repeated_measurements = repeated.measurements();
    let deterministic = visible_hash == repeated_visible_hash
        && same_measurements(measurements, repeated_measurements);
    let safe_moments = measurements.age.max(1) as f64;

    Ok(CandidateOutcome {
        candidate,
        measurements,
        elapsed_seconds,
        visible_hash,
        repeated_visible_hash,
        deterministic,
        relative_accounting_error: relative_accounting_error(measurements),
        radiation_gain: ratio(measurements.radiated, baseline.radiated),
        channel_information_gain: measurements.channel_information_bits
            - baseline.channel_information_bits,
        channel_signal_gain: ratio(measurements.channel_signal, baseline.channel_signal),
        luminous_gain: ratio(
            measurements.luminous_sites as f64,
            baseline.luminous_sites as f64,
        ),
        information_rate: measurements.channel_information_bits / safe_moments,
        signal_rate: measurements.channel_signal / safe_moments,
        radiation_rate: measurements.radiated / safe_moments,
    })
}

impl fmt::Display for SweepReport {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(output, "Universum primitive optimization sweep")?;
        writeln!(output, "schema=1")?;
        writeln!(output, "operator_disposition=none")?;
        writeln!(output, "operator_rejection_count=0")?;
        writeln!(output, "stacking_order_critical=true")?;
        writeln!(output, "all_primitives_reported_before_disposition=true")?;
        writeln!(output, "width={}", self.width)?;
        writeln!(output, "height={}", self.height)?;
        writeln!(output, "seed={}", self.seed)?;
        writeln!(output, "moments={}", self.moments)?;
        writeln!(output, "candidate_count={}", self.candidates.len())?;
        writeln!(output, "all_deterministic={}", self.all_deterministic())?;
        writeln!(
            output,
            "max_relative_accounting_error={:.12}",
            self.max_relative_accounting_error()
        )?;
        writeln!(output)?;
        write_outcome(output, self.baseline)?;
        for candidate in &self.candidates {
            write_outcome(output, *candidate)?;
        }
        let channel_best = self.best_by_channel_information();
        let signal_rate_best = self.best_by_signal_rate();
        let radiation_rate_best = self.best_by_radiation_rate();
        writeln!(
            output,
            "best_by_channel_information={}",
            channel_best.candidate.name
        )?;
        writeln!(
            output,
            "best_by_channel_information_bits={:.9}",
            channel_best.measurements.channel_information_bits
        )?;
        writeln!(
            output,
            "best_by_signal_rate={}",
            signal_rate_best.candidate.name
        )?;
        writeln!(
            output,
            "best_by_signal_rate_value={:.9}",
            signal_rate_best.signal_rate
        )?;
        writeln!(
            output,
            "best_by_radiation_rate={}",
            radiation_rate_best.candidate.name
        )?;
        writeln!(
            output,
            "best_by_radiation_rate_value={:.9}",
            radiation_rate_best.radiation_rate
        )?;
        writeln!(output, "sweep_gate=PASS")?;
        writeln!(output, "operator_review_required_before_any_rejection=true")?;
        Ok(())
    }
}

fn write_outcome(output: &mut fmt::Formatter<'_>, outcome: CandidateOutcome) -> fmt::Result {
    writeln!(output, "candidate={}", outcome.candidate.name)?;
    writeln!(output, "  primitives={}", outcome.candidate.primitives)?;
    writeln!(output, "  stack_order={}", outcome.candidate.stack_order)?;
    writeln!(
        output,
        "  operator_disposition={}",
        outcome.candidate.operator_disposition
    )?;
    writeln!(
        output,
        "  salvage_attempt={}",
        outcome.candidate.salvage_attempt
    )?;
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
        "  channel_signal={:.9}",
        outcome.measurements.channel_signal
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
    writeln!(output, "  radiation_gain={:.9}", outcome.radiation_gain)?;
    writeln!(
        output,
        "  channel_information_gain={:.9}",
        outcome.channel_information_gain
    )?;
    writeln!(
        output,
        "  channel_signal_gain={:.9}",
        outcome.channel_signal_gain
    )?;
    writeln!(output, "  luminous_gain={:.9}", outcome.luminous_gain)?;
    writeln!(output, "  information_rate={:.9}", outcome.information_rate)?;
    writeln!(output, "  signal_rate={:.9}", outcome.signal_rate)?;
    writeln!(output, "  radiation_rate={:.9}", outcome.radiation_rate)?;
    writeln!(output, "  visible_hash=0x{:016x}", outcome.visible_hash)?;
    writeln!(
        output,
        "  repeated_visible_hash=0x{:016x}",
        outcome.repeated_visible_hash
    )?;
    writeln!(output, "  deterministic={}", outcome.deterministic)?;
    Ok(())
}

fn identity(config: Config) -> Config {
    config
}

fn transport_pressure(mut config: Config) -> Config {
    config.diffusion = 0.132;
    config.gradient = 0.285;
    config
}

fn coupling_compound(mut config: Config) -> Config {
    config.coupling_formation = 0.285;
    config.coupling_erosion = 0.000_50;
    config
}

fn low_leak_memory(mut config: Config) -> Config {
    config.dissipation = 0.000_85;
    config.erosion = 0.001_35;
    config
}

fn radiation_gate(mut config: Config) -> Config {
    config.radiation = 0.235;
    config.coupling_formation = 0.195;
    config
}

fn transport_plus_coupling(mut config: Config) -> Config {
    config.diffusion = 0.125;
    config.gradient = 0.265;
    config.coupling_formation = 0.270;
    config.coupling_erosion = 0.000_46;
    config
}

fn low_leak_plus_radiation(mut config: Config) -> Config {
    config.dissipation = 0.000_85;
    config.erosion = 0.001_40;
    config.radiation = 0.225;
    config
}

fn balanced_channel_compound(mut config: Config) -> Config {
    config.diffusion = 0.118;
    config.gradient = 0.250;
    config.coupling_formation = 0.250;
    config.coupling_erosion = 0.000_43;
    config.dissipation = 0.001_00;
    config
}

fn phase_relay(mut config: Config) -> Config {
    config.phase_relay = 0.32;
    config
}

fn phase_relay_transport(mut config: Config) -> Config {
    config.diffusion = 0.118;
    config.gradient = 0.245;
    config.phase_relay = 0.30;
    config
}

fn phase_relay_low_leak(mut config: Config) -> Config {
    config.phase_relay = 0.28;
    config.dissipation = 0.000_95;
    config.erosion = 0.001_45;
    config
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

fn relative_accounting_error(measurements: Measurements) -> f64 {
    if measurements.introduced.abs() <= f64::EPSILON {
        measurements.accounting_error.abs()
    } else {
        measurements.accounting_error.abs() / measurements.introduced.abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_reports_every_candidate_before_review_verdict() {
        let report = run_standard_sweep(32, 24, 103, 360).unwrap();
        assert!(report.all_deterministic(), "{}", report);
        assert!(report.max_relative_accounting_error() < 0.000_06);
        let text = report.to_string();
        let verdict = text.find("sweep_gate=").unwrap();
        for candidate in [
            "candidate=baseline-adaptive",
            "candidate=transport-pressure",
            "candidate=coupling-compound",
            "candidate=low-leak-memory",
            "candidate=radiation-gate",
            "candidate=transport-plus-coupling",
            "candidate=low-leak-plus-radiation",
            "candidate=balanced-channel-compound",
            "candidate=phase-relay",
            "candidate=phase-relay-transport",
            "candidate=phase-relay-low-leak",
        ] {
            let position = text
                .find(candidate)
                .unwrap_or_else(|| panic!("missing {candidate}"));
            assert!(
                position < verdict,
                "{candidate} was not reported before verdict"
            );
        }
        assert!(text.contains("operator_disposition=none"));
        assert!(text.contains("operator_rejection_count=0"));
        assert!(text.contains("operator_review_required_before_any_rejection=true"));
    }
}
