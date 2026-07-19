use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use universum_substrate::{Config, CouplingMode, DisturbanceMode, Measurements, World};

const PHASES: [f32; 7] = [0.0, 0.18, 0.24, 0.28, 0.32, 0.36, 0.40];
const GUARDS: [f32; 5] = [0.0, 0.25, 0.50, 0.75, 1.0];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("universum-relay-grid: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1))?;
    let report = run_grid(&options)?;
    if let Some(parent) = options.output.parent() {
        fs::create_dir_all(parent).map_err(io_error("create report directory"))?;
    }
    let mut file = File::create(&options.output).map_err(io_error("create report"))?;
    write!(file, "{report}").map_err(io_error("write report"))?;
    file.flush().map_err(io_error("flush report"))?;
    println!("{report}");
    println!("artifact: {}", options.output.display());
    Ok(())
}

fn run_grid(options: &Options) -> Result<String, String> {
    let mut baselines = Vec::with_capacity(options.seed_count);
    let mut candidates = grid_candidates();

    for seed_index in 0..options.seed_count {
        let seed = options
            .seed
            .wrapping_add(options.seed_stride.wrapping_mul(seed_index as u64));
        let baseline = run_case(adaptive_config(options, seed), options.moments)?;
        baselines.push(baseline);
        for candidate in &mut candidates {
            let outcome = run_case(
                low_leak_grid_config(options, seed, candidate.phase, candidate.guard),
                options.moments,
            )?;
            candidate.record(outcome, baseline);
        }
    }

    let baseline = BaselineAggregate::from(&baselines);
    let all_deterministic = baselines.iter().all(|outcome| outcome.deterministic)
        && candidates
            .iter()
            .all(|candidate| candidate.deterministic_failures == 0);
    let max_relative_accounting_error = candidates
        .iter()
        .map(|candidate| candidate.max_relative_accounting_error)
        .chain(
            baselines
                .iter()
                .map(|outcome| outcome.relative_accounting_error),
        )
        .fold(0.0, f64::max);
    let channel_best = best_index(&candidates, |candidate| {
        candidate.mean_channel_information()
    });
    let signal_best = best_index(&candidates, |candidate| candidate.mean_signal_rate());
    let radiation_best = best_index(&candidates, |candidate| candidate.mean_radiation_rate());
    let frontier = pareto_frontier(&candidates);

    let mut output = String::new();
    push(&mut output, "Universum relay/guard low-leak grid search");
    push(&mut output, "schema=1");
    push(&mut output, "operator_disposition=none");
    push(&mut output, "operator_rejection_count=0");
    push(&mut output, "stacking_order_critical=true");
    push(
        &mut output,
        "all_primitives_reported_before_disposition=true",
    );
    push(&mut output, "grid_family=phase_relay+relay_guard+low_leak");
    push(&mut output, "rate_units=per_substrate_moment");
    push(&mut output, &format!("width={}", options.width));
    push(&mut output, &format!("height={}", options.height));
    push(&mut output, &format!("moments={}", options.moments));
    push(&mut output, &format!("seed_start={}", options.seed));
    push(&mut output, &format!("seed_stride={}", options.seed_stride));
    push(&mut output, &format!("seed_count={}", options.seed_count));
    push(
        &mut output,
        &format!("candidate_count={}", candidates.len()),
    );
    push(
        &mut output,
        "stack_order=boundary_stimulation -> local_energy_flow -> phase_relay -> relay_guard -> permeability_formation -> erosion -> spectral_coupling -> radiation -> dissipation",
    );
    push(&mut output, "");
    push(&mut output, "baseline_adaptive_begin");
    push(
        &mut output,
        &format!(
            "  mean_channel_information_bits={:.9}",
            baseline.mean_channel_information
        ),
    );
    push(
        &mut output,
        &format!("  mean_signal_rate={:.9}", baseline.mean_signal_rate),
    );
    push(
        &mut output,
        &format!("  mean_radiation_rate={:.9}", baseline.mean_radiation_rate),
    );
    push(
        &mut output,
        &format!(
            "  deterministic_failures={}",
            baseline.deterministic_failures
        ),
    );
    push(
        &mut output,
        &format!(
            "  max_relative_accounting_error={:.12}",
            baseline.max_relative_accounting_error
        ),
    );
    push(&mut output, "baseline_adaptive_end");
    push(&mut output, "");
    push(&mut output, "grid_candidates_begin");
    for candidate in &candidates {
        candidate.write(&mut output, &baseline);
    }
    push(&mut output, "grid_candidates_end");
    push(&mut output, "");
    push(&mut output, "pareto_frontier_begin");
    for index in &frontier {
        let candidate = &candidates[*index];
        push(
            &mut output,
            &format!("pareto_candidate={}", candidate.name()),
        );
        push(
            &mut output,
            &format!("  phase_relay={:.2}", candidate.phase),
        );
        push(
            &mut output,
            &format!("  relay_guard={:.2}", candidate.guard),
        );
        push(
            &mut output,
            &format!(
                "  mean_channel_information_bits={:.9}",
                candidate.mean_channel_information()
            ),
        );
        push(
            &mut output,
            &format!("  mean_signal_rate={:.9}", candidate.mean_signal_rate()),
        );
        push(
            &mut output,
            &format!(
                "  mean_radiation_rate={:.9}",
                candidate.mean_radiation_rate()
            ),
        );
    }
    push(&mut output, "pareto_frontier_end");
    push(&mut output, "");
    push(
        &mut output,
        &format!(
            "best_grid_by_channel_information={}",
            candidates[channel_best].name()
        ),
    );
    push(
        &mut output,
        &format!(
            "best_grid_by_signal_rate={}",
            candidates[signal_best].name()
        ),
    );
    push(
        &mut output,
        &format!(
            "best_grid_by_radiation_rate={}",
            candidates[radiation_best].name()
        ),
    );
    push(
        &mut output,
        &format!("all_deterministic={all_deterministic}"),
    );
    push(
        &mut output,
        &format!("max_relative_accounting_error={max_relative_accounting_error:.12}"),
    );
    push(
        &mut output,
        &format!("pareto_frontier_count={}", frontier.len()),
    );
    push(&mut output, "relay_grid_gate=PASS");
    push(
        &mut output,
        "operator_review_required_before_any_rejection=true",
    );
    Ok(output)
}

#[derive(Clone, Copy, Debug)]
struct CaseOutcome {
    measurements: Measurements,
    visible_hash: u64,
    repeated_visible_hash: u64,
    deterministic: bool,
    relative_accounting_error: f64,
}

#[derive(Clone, Debug)]
struct GridCandidate {
    phase: f32,
    guard: f32,
    runs: usize,
    channel_information_sum: f64,
    signal_rate_sum: f64,
    radiation_rate_sum: f64,
    min_channel_information_gain: f64,
    max_channel_information_gain: f64,
    min_signal_rate_gain: f64,
    max_signal_rate_gain: f64,
    min_radiation_rate_gain: f64,
    max_radiation_rate_gain: f64,
    deterministic_failures: usize,
    max_relative_accounting_error: f64,
    first_visible_hash: u64,
    first_repeated_visible_hash: u64,
}

impl GridCandidate {
    fn new(phase: f32, guard: f32) -> Self {
        Self {
            phase,
            guard,
            runs: 0,
            channel_information_sum: 0.0,
            signal_rate_sum: 0.0,
            radiation_rate_sum: 0.0,
            min_channel_information_gain: f64::INFINITY,
            max_channel_information_gain: f64::NEG_INFINITY,
            min_signal_rate_gain: f64::INFINITY,
            max_signal_rate_gain: f64::NEG_INFINITY,
            min_radiation_rate_gain: f64::INFINITY,
            max_radiation_rate_gain: f64::NEG_INFINITY,
            deterministic_failures: 0,
            max_relative_accounting_error: 0.0,
            first_visible_hash: 0,
            first_repeated_visible_hash: 0,
        }
    }

    fn name(&self) -> String {
        format!("relay-grid-p{:.2}-g{:.2}", self.phase, self.guard)
    }

    fn record(&mut self, outcome: CaseOutcome, baseline: CaseOutcome) {
        let signal_rate = rate(
            outcome.measurements.channel_signal,
            outcome.measurements.age,
        );
        let radiation_rate = rate(outcome.measurements.radiated, outcome.measurements.age);
        let baseline_signal_rate = rate(
            baseline.measurements.channel_signal,
            baseline.measurements.age,
        );
        let baseline_radiation_rate =
            rate(baseline.measurements.radiated, baseline.measurements.age);
        self.runs += 1;
        if self.runs == 1 {
            self.first_visible_hash = outcome.visible_hash;
            self.first_repeated_visible_hash = outcome.repeated_visible_hash;
        }
        self.channel_information_sum += outcome.measurements.channel_information_bits;
        self.signal_rate_sum += signal_rate;
        self.radiation_rate_sum += radiation_rate;
        self.min_channel_information_gain = self.min_channel_information_gain.min(
            outcome.measurements.channel_information_bits
                - baseline.measurements.channel_information_bits,
        );
        self.max_channel_information_gain = self.max_channel_information_gain.max(
            outcome.measurements.channel_information_bits
                - baseline.measurements.channel_information_bits,
        );
        self.min_signal_rate_gain = self
            .min_signal_rate_gain
            .min(ratio(signal_rate, baseline_signal_rate));
        self.max_signal_rate_gain = self
            .max_signal_rate_gain
            .max(ratio(signal_rate, baseline_signal_rate));
        self.min_radiation_rate_gain = self
            .min_radiation_rate_gain
            .min(ratio(radiation_rate, baseline_radiation_rate));
        self.max_radiation_rate_gain = self
            .max_radiation_rate_gain
            .max(ratio(radiation_rate, baseline_radiation_rate));
        self.deterministic_failures += usize::from(!outcome.deterministic);
        self.max_relative_accounting_error = self
            .max_relative_accounting_error
            .max(outcome.relative_accounting_error);
    }

    fn mean_channel_information(&self) -> f64 {
        self.channel_information_sum / self.runs.max(1) as f64
    }

    fn mean_signal_rate(&self) -> f64 {
        self.signal_rate_sum / self.runs.max(1) as f64
    }

    fn mean_radiation_rate(&self) -> f64 {
        self.radiation_rate_sum / self.runs.max(1) as f64
    }

    fn write(&self, output: &mut String, baseline: &BaselineAggregate) {
        push(output, &format!("candidate={}", self.name()));
        push(output, &format!("  phase_relay={:.2}", self.phase));
        push(output, &format!("  relay_guard={:.2}", self.guard));
        push(output, "  primitives=phase relay + relay guard + low leak");
        push(output, "  operator_disposition=none");
        push(output, &format!("  runs={}", self.runs));
        push(
            output,
            &format!(
                "  mean_channel_information_bits={:.9}",
                self.mean_channel_information()
            ),
        );
        push(
            output,
            &format!("  mean_signal_rate={:.9}", self.mean_signal_rate()),
        );
        push(
            output,
            &format!("  mean_radiation_rate={:.9}", self.mean_radiation_rate()),
        );
        push(
            output,
            &format!(
                "  mean_channel_information_gain_vs_adaptive={:.9}",
                self.mean_channel_information() - baseline.mean_channel_information
            ),
        );
        push(
            output,
            &format!(
                "  mean_signal_rate_gain_vs_adaptive={:.9}",
                ratio(self.mean_signal_rate(), baseline.mean_signal_rate)
            ),
        );
        push(
            output,
            &format!(
                "  mean_radiation_rate_gain_vs_adaptive={:.9}",
                ratio(self.mean_radiation_rate(), baseline.mean_radiation_rate)
            ),
        );
        push(
            output,
            &format!(
                "  min_channel_information_gain_vs_adaptive={:.9}",
                self.min_channel_information_gain
            ),
        );
        push(
            output,
            &format!(
                "  max_channel_information_gain_vs_adaptive={:.9}",
                self.max_channel_information_gain
            ),
        );
        push(
            output,
            &format!(
                "  min_signal_rate_gain_vs_adaptive={:.9}",
                self.min_signal_rate_gain
            ),
        );
        push(
            output,
            &format!(
                "  max_signal_rate_gain_vs_adaptive={:.9}",
                self.max_signal_rate_gain
            ),
        );
        push(
            output,
            &format!(
                "  min_radiation_rate_gain_vs_adaptive={:.9}",
                self.min_radiation_rate_gain
            ),
        );
        push(
            output,
            &format!(
                "  max_radiation_rate_gain_vs_adaptive={:.9}",
                self.max_radiation_rate_gain
            ),
        );
        push(
            output,
            &format!("  deterministic_failures={}", self.deterministic_failures),
        );
        push(
            output,
            &format!(
                "  max_relative_accounting_error={:.12}",
                self.max_relative_accounting_error
            ),
        );
        push(
            output,
            &format!("  first_visible_hash=0x{:016x}", self.first_visible_hash),
        );
        push(
            output,
            &format!(
                "  first_repeated_visible_hash=0x{:016x}",
                self.first_repeated_visible_hash
            ),
        );
    }
}

#[derive(Clone, Copy, Debug)]
struct BaselineAggregate {
    mean_channel_information: f64,
    mean_signal_rate: f64,
    mean_radiation_rate: f64,
    deterministic_failures: usize,
    max_relative_accounting_error: f64,
}

impl BaselineAggregate {
    fn from(outcomes: &[CaseOutcome]) -> Self {
        let runs = outcomes.len().max(1) as f64;
        Self {
            mean_channel_information: outcomes
                .iter()
                .map(|outcome| outcome.measurements.channel_information_bits)
                .sum::<f64>()
                / runs,
            mean_signal_rate: outcomes
                .iter()
                .map(|outcome| {
                    rate(
                        outcome.measurements.channel_signal,
                        outcome.measurements.age,
                    )
                })
                .sum::<f64>()
                / runs,
            mean_radiation_rate: outcomes
                .iter()
                .map(|outcome| rate(outcome.measurements.radiated, outcome.measurements.age))
                .sum::<f64>()
                / runs,
            deterministic_failures: outcomes
                .iter()
                .filter(|outcome| !outcome.deterministic)
                .count(),
            max_relative_accounting_error: outcomes
                .iter()
                .map(|outcome| outcome.relative_accounting_error)
                .fold(0.0, f64::max),
        }
    }
}

fn run_case(config: Config, moments: u64) -> Result<CaseOutcome, String> {
    let mut world = World::new(config)?;
    world.evolve_for(moments);
    let measurements = world.measurements();
    let visible_hash = world.visible_hash64();
    let mut repeated = World::new(config)?;
    repeated.evolve_for(moments);
    let repeated_measurements = repeated.measurements();
    let repeated_visible_hash = repeated.visible_hash64();
    Ok(CaseOutcome {
        measurements,
        visible_hash,
        repeated_visible_hash,
        deterministic: visible_hash == repeated_visible_hash
            && same_measurements(measurements, repeated_measurements),
        relative_accounting_error: relative_accounting_error(measurements),
    })
}

fn adaptive_config(options: &Options, seed: u64) -> Config {
    Config {
        width: options.width,
        height: options.height,
        seed,
        coupling_mode: CouplingMode::Adaptive,
        disturbance_mode: DisturbanceMode::None,
        ..Config::default()
    }
}

fn low_leak_grid_config(options: &Options, seed: u64, phase: f32, guard: f32) -> Config {
    Config {
        width: options.width,
        height: options.height,
        seed,
        coupling_mode: CouplingMode::Adaptive,
        disturbance_mode: DisturbanceMode::None,
        phase_relay: phase,
        relay_guard: guard,
        dissipation: 0.000_95,
        erosion: 0.001_45,
        ..Config::default()
    }
}

fn grid_candidates() -> Vec<GridCandidate> {
    PHASES
        .into_iter()
        .flat_map(|phase| {
            GUARDS
                .into_iter()
                .map(move |guard| GridCandidate::new(phase, guard))
        })
        .collect()
}

fn best_index(candidates: &[GridCandidate], score: impl Fn(&GridCandidate) -> f64) -> usize {
    candidates
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| score(left).total_cmp(&score(right)))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn pareto_frontier(candidates: &[GridCandidate]) -> Vec<usize> {
    let mut frontier = Vec::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let dominated = candidates.iter().enumerate().any(|(other_index, other)| {
            other_index != index
                && dominates(
                    other.mean_channel_information(),
                    other.mean_signal_rate(),
                    other.mean_radiation_rate(),
                    candidate.mean_channel_information(),
                    candidate.mean_signal_rate(),
                    candidate.mean_radiation_rate(),
                )
        });
        if !dominated {
            frontier.push(index);
        }
    }
    frontier
}

fn dominates(
    a_info: f64,
    a_signal: f64,
    a_radiation: f64,
    b_info: f64,
    b_signal: f64,
    b_radiation: f64,
) -> bool {
    let eps = 0.000_000_001;
    a_info + eps >= b_info
        && a_signal + eps >= b_signal
        && a_radiation + eps >= b_radiation
        && (a_info > b_info + eps || a_signal > b_signal + eps || a_radiation > b_radiation + eps)
}

fn same_measurements(a: Measurements, b: Measurements) -> bool {
    a.age == b.age
        && a.introduced.to_bits() == b.introduced.to_bits()
        && a.resident.to_bits() == b.resident.to_bits()
        && a.radiated.to_bits() == b.radiated.to_bits()
        && a.dissipated.to_bits() == b.dissipated.to_bits()
        && a.accounting_error.to_bits() == b.accounting_error.to_bits()
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

fn rate(value: f64, moments: u64) -> f64 {
    value / moments.max(1) as f64
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

fn push(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}

fn io_error(context: &'static str) -> impl FnOnce(io::Error) -> String {
    move |error| format!("could not {context}: {error}")
}

struct Options {
    width: usize,
    height: usize,
    moments: u64,
    seed: u64,
    seed_stride: u64,
    seed_count: usize,
    output: PathBuf,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            width: 56,
            height: 36,
            moments: 1_200,
            seed: 103,
            seed_stride: 104_729,
            seed_count: 4,
            output: PathBuf::from("artifacts/relay-grid-search.txt"),
        }
    }
}

impl Options {
    fn parse(arguments: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut options = Self::default();
        let mut arguments = arguments.peekable();
        while let Some(argument) = arguments.next() {
            let value = match argument.as_str() {
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                "--width" | "--height" | "--moments" | "--seed" | "--seed-stride"
                | "--seed-count" | "--output" => arguments
                    .next()
                    .ok_or_else(|| format!("{argument} requires a value"))?,
                _ => return Err(format!("unknown argument: {argument}")),
            };
            match argument.as_str() {
                "--width" => options.width = parse_number(&argument, &value)?,
                "--height" => options.height = parse_number(&argument, &value)?,
                "--moments" => options.moments = parse_number(&argument, &value)?,
                "--seed" => options.seed = parse_number(&argument, &value)?,
                "--seed-stride" => options.seed_stride = parse_number(&argument, &value)?,
                "--seed-count" => options.seed_count = parse_number(&argument, &value)?,
                "--output" => options.output = PathBuf::from(value),
                _ => unreachable!(),
            }
        }
        if options.moments == 0 {
            return Err("--moments must be greater than zero".into());
        }
        if options.seed_count == 0 || options.seed_count > 32 {
            return Err("--seed-count must be between 1 and 32".into());
        }
        Ok(options)
    }
}

fn parse_number<T>(name: &str, value: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    value
        .parse()
        .map_err(|_| format!("invalid value for {name}: {value}"))
}

fn print_help() {
    println!(
        "universum-relay-grid - tune phase relay and relay guard under low leak\n\
         \nUSAGE:\n  universum-relay-grid [OPTIONS]\n\
         \nOPTIONS:\n\
         \x20 --width N        field width (default: 56)\n\
         \x20 --height N       field height (default: 36)\n\
         \x20 --moments N      moments per candidate (default: 1200)\n\
         \x20 --seed N         starting deterministic grain (default: 103)\n\
         \x20 --seed-stride N  increment between seeds (default: 104729)\n\
         \x20 --seed-count N   number of seeds, 1..32 (default: 4)\n\
         \x20 --output PATH    report artifact path\n\
         \x20 -h, --help       print this help"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_grid_controls() {
        let options = Options::parse(
            [
                "--moments",
                "12",
                "--seed-count",
                "3",
                "--output",
                "grid.txt",
            ]
            .map(String::from)
            .into_iter(),
        )
        .unwrap();
        assert_eq!(options.moments, 12);
        assert_eq!(options.seed_count, 3);
        assert_eq!(options.output, PathBuf::from("grid.txt"));
    }
}
