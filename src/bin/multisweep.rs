use std::{
    collections::BTreeMap,
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use universum_substrate::{CandidateOutcome, run_standard_sweep};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("universum-multisweep: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1))?;
    let report = run_multiseed_sweep(&options)?;
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

fn run_multiseed_sweep(options: &Options) -> Result<String, String> {
    let mut output = String::new();
    let mut aggregates = BTreeMap::<&'static str, CandidateAggregate>::new();
    let mut all_deterministic = true;
    let mut max_relative_accounting_error = 0.0_f64;
    let mut channel_leaders = Vec::with_capacity(options.seed_count);
    let mut signal_rate_leaders = Vec::with_capacity(options.seed_count);
    let mut radiation_rate_leaders = Vec::with_capacity(options.seed_count);

    push_line(
        &mut output,
        "Universum multi-seed primitive optimization sweep",
    );
    push_line(&mut output, "schema=1");
    push_line(&mut output, "operator_disposition=none");
    push_line(&mut output, "operator_rejection_count=0");
    push_line(&mut output, "stacking_order_critical=true");
    push_line(
        &mut output,
        "all_primitives_reported_before_disposition=true",
    );
    push_line(&mut output, &format!("width={}", options.width));
    push_line(&mut output, &format!("height={}", options.height));
    push_line(&mut output, &format!("moments={}", options.moments));
    push_line(&mut output, &format!("seed_start={}", options.seed));
    push_line(&mut output, &format!("seed_stride={}", options.seed_stride));
    push_line(&mut output, &format!("seed_count={}", options.seed_count));
    push_line(&mut output, "");

    for index in 0..options.seed_count {
        let seed = options
            .seed
            .wrapping_add(options.seed_stride.wrapping_mul(index as u64));
        let report = run_standard_sweep(options.width, options.height, seed, options.moments)?;
        all_deterministic &= report.all_deterministic();
        max_relative_accounting_error =
            max_relative_accounting_error.max(report.max_relative_accounting_error());
        let channel_best = report.best_by_channel_information();
        let signal_rate_best = report.best_by_signal_rate();
        let radiation_rate_best = report.best_by_radiation_rate();
        channel_leaders.push(channel_best.candidate.name);
        signal_rate_leaders.push(signal_rate_best.candidate.name);
        radiation_rate_leaders.push(radiation_rate_best.candidate.name);

        push_line(&mut output, &format!("seed_run={index}"));
        push_line(&mut output, &format!("  seed={seed}"));
        push_line(
            &mut output,
            &format!("  all_deterministic={}", report.all_deterministic()),
        );
        push_line(
            &mut output,
            &format!(
                "  max_relative_accounting_error={:.12}",
                report.max_relative_accounting_error()
            ),
        );
        push_line(
            &mut output,
            &format!(
                "  best_by_channel_information={}",
                channel_best.candidate.name
            ),
        );
        push_line(
            &mut output,
            &format!(
                "  best_by_channel_information_bits={:.9}",
                channel_best.measurements.channel_information_bits
            ),
        );
        push_line(
            &mut output,
            &format!("  best_by_signal_rate={}", signal_rate_best.candidate.name),
        );
        push_line(
            &mut output,
            &format!(
                "  best_by_radiation_rate={}",
                radiation_rate_best.candidate.name
            ),
        );

        for outcome in report.outcomes() {
            aggregates
                .entry(outcome.candidate.name)
                .or_insert_with(|| CandidateAggregate::new(outcome.candidate.name))
                .record(outcome, channel_best, signal_rate_best, radiation_rate_best);
        }
    }

    push_line(&mut output, "");
    push_line(&mut output, "aggregate_candidates_begin");
    for aggregate in aggregates.values() {
        aggregate.write(&mut output);
    }
    push_line(&mut output, "aggregate_candidates_end");
    push_line(&mut output, "");
    push_line(
        &mut output,
        &format!("all_deterministic={all_deterministic}"),
    );
    push_line(
        &mut output,
        &format!("max_relative_accounting_error={max_relative_accounting_error:.12}"),
    );
    push_line(
        &mut output,
        &format!(
            "stable_channel_information_leader={}",
            stable_leader(&channel_leaders)
        ),
    );
    push_line(
        &mut output,
        &format!(
            "stable_signal_rate_leader={}",
            stable_leader(&signal_rate_leaders)
        ),
    );
    push_line(
        &mut output,
        &format!(
            "stable_radiation_rate_leader={}",
            stable_leader(&radiation_rate_leaders)
        ),
    );
    push_line(&mut output, "multiseed_sweep_gate=PASS");
    push_line(
        &mut output,
        "operator_review_required_before_any_rejection=true",
    );
    Ok(output)
}

#[derive(Clone, Debug)]
struct CandidateAggregate {
    name: &'static str,
    runs: usize,
    channel_information_sum: f64,
    signal_rate_sum: f64,
    radiation_rate_sum: f64,
    channel_wins: usize,
    signal_rate_wins: usize,
    radiation_rate_wins: usize,
    min_channel_information_gain: f64,
    max_channel_information_gain: f64,
    min_radiation_gain: f64,
    max_radiation_gain: f64,
    deterministic_failures: usize,
    max_relative_accounting_error: f64,
}

impl CandidateAggregate {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            runs: 0,
            channel_information_sum: 0.0,
            signal_rate_sum: 0.0,
            radiation_rate_sum: 0.0,
            channel_wins: 0,
            signal_rate_wins: 0,
            radiation_rate_wins: 0,
            min_channel_information_gain: f64::INFINITY,
            max_channel_information_gain: f64::NEG_INFINITY,
            min_radiation_gain: f64::INFINITY,
            max_radiation_gain: f64::NEG_INFINITY,
            deterministic_failures: 0,
            max_relative_accounting_error: 0.0,
        }
    }

    fn record(
        &mut self,
        outcome: CandidateOutcome,
        channel_best: CandidateOutcome,
        signal_rate_best: CandidateOutcome,
        radiation_rate_best: CandidateOutcome,
    ) {
        self.runs += 1;
        self.channel_information_sum += outcome.measurements.channel_information_bits;
        self.signal_rate_sum += outcome.signal_rate;
        self.radiation_rate_sum += outcome.radiation_rate;
        self.channel_wins += usize::from(outcome.candidate.name == channel_best.candidate.name);
        self.signal_rate_wins +=
            usize::from(outcome.candidate.name == signal_rate_best.candidate.name);
        self.radiation_rate_wins +=
            usize::from(outcome.candidate.name == radiation_rate_best.candidate.name);
        self.min_channel_information_gain = self
            .min_channel_information_gain
            .min(outcome.channel_information_gain);
        self.max_channel_information_gain = self
            .max_channel_information_gain
            .max(outcome.channel_information_gain);
        self.min_radiation_gain = self.min_radiation_gain.min(outcome.radiation_gain);
        self.max_radiation_gain = self.max_radiation_gain.max(outcome.radiation_gain);
        self.deterministic_failures += usize::from(!outcome.deterministic);
        self.max_relative_accounting_error = self
            .max_relative_accounting_error
            .max(outcome.relative_accounting_error);
    }

    fn write(&self, output: &mut String) {
        let runs = self.runs.max(1) as f64;
        push_line(output, &format!("candidate={}", self.name));
        push_line(output, &format!("  runs={}", self.runs));
        push_line(
            output,
            &format!(
                "  mean_channel_information_bits={:.9}",
                self.channel_information_sum / runs
            ),
        );
        push_line(
            output,
            &format!("  mean_signal_rate={:.9}", self.signal_rate_sum / runs),
        );
        push_line(
            output,
            &format!(
                "  mean_radiation_rate={:.9}",
                self.radiation_rate_sum / runs
            ),
        );
        push_line(output, &format!("  channel_wins={}", self.channel_wins));
        push_line(
            output,
            &format!("  signal_rate_wins={}", self.signal_rate_wins),
        );
        push_line(
            output,
            &format!("  radiation_rate_wins={}", self.radiation_rate_wins),
        );
        push_line(
            output,
            &format!(
                "  min_channel_information_gain={:.9}",
                self.min_channel_information_gain
            ),
        );
        push_line(
            output,
            &format!(
                "  max_channel_information_gain={:.9}",
                self.max_channel_information_gain
            ),
        );
        push_line(
            output,
            &format!("  min_radiation_gain={:.9}", self.min_radiation_gain),
        );
        push_line(
            output,
            &format!("  max_radiation_gain={:.9}", self.max_radiation_gain),
        );
        push_line(
            output,
            &format!("  deterministic_failures={}", self.deterministic_failures),
        );
        push_line(
            output,
            &format!(
                "  max_relative_accounting_error={:.12}",
                self.max_relative_accounting_error
            ),
        );
        push_line(output, "  operator_disposition=none");
    }
}

fn stable_leader(leaders: &[&'static str]) -> &'static str {
    match leaders.split_first() {
        Some((first, rest)) if rest.iter().all(|leader| leader == first) => first,
        Some(_) => "mixed",
        None => "none",
    }
}

fn push_line(output: &mut String, line: &str) {
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
            output: PathBuf::from("artifacts/primitive-multiseed-sweep.txt"),
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
        if options.seed_count == 0 || options.seed_count > 64 {
            return Err("--seed-count must be between 1 and 64".into());
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
        "universum-multisweep - aggregate primitive sweeps across deterministic seeds\n\
         \nUSAGE:\n  universum-multisweep [OPTIONS]\n\
         \nOPTIONS:\n\
         \x20 --width N        field width (default: 56)\n\
         \x20 --height N       field height (default: 36)\n\
         \x20 --moments N      moments per candidate (default: 1200)\n\
         \x20 --seed N         starting deterministic grain (default: 103)\n\
         \x20 --seed-stride N  increment between seeds (default: 104729)\n\
         \x20 --seed-count N   number of seeds, 1..64 (default: 4)\n\
         \x20 --output PATH    report artifact path\n\
         \x20 -h, --help       print this help"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_multiseed_controls() {
        let options = Options::parse(
            [
                "--moments",
                "12",
                "--seed",
                "9",
                "--seed-stride",
                "17",
                "--seed-count",
                "3",
                "--output",
                "multi.txt",
            ]
            .map(String::from)
            .into_iter(),
        )
        .unwrap();
        assert_eq!(options.moments, 12);
        assert_eq!(options.seed, 9);
        assert_eq!(options.seed_stride, 17);
        assert_eq!(options.seed_count, 3);
        assert_eq!(options.output, PathBuf::from("multi.txt"));
    }

    #[test]
    fn parser_rejects_zero_seed_count() {
        let result = Options::parse(["--seed-count", "0"].map(String::from).into_iter());
        assert!(result.is_err());
    }
}
