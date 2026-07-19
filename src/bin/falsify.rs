use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use universum_substrate::{
    FalsificationReport, FalsificationThresholds, run_standard_falsification,
};

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("universum-falsify: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<bool, String> {
    let options = Options::parse(env::args().skip(1))?;
    let (report, passed) = if options.seed_count == 1 {
        let report = run_one(&options, options.seed)?;
        (report.to_string(), report.passed())
    } else {
        run_cohort(&options)?
    };
    if let Some(parent) = options.output.parent() {
        fs::create_dir_all(parent).map_err(io_error("create report directory"))?;
    }
    let mut file = File::create(&options.output).map_err(io_error("create report"))?;
    write!(file, "{report}").map_err(io_error("write report"))?;
    file.flush().map_err(io_error("flush report"))?;

    println!("{report}");
    println!("artifact: {}", options.output.display());
    Ok(passed)
}

fn run_one(options: &Options, seed: u64) -> Result<FalsificationReport, String> {
    run_standard_falsification(
        options.width,
        options.height,
        seed,
        options.moments,
        FalsificationThresholds::default(),
    )
}

fn run_cohort(options: &Options) -> Result<(String, bool), String> {
    let mut output = String::new();
    let mut passed = true;
    let mut max_relative_accounting_error = 0.0_f64;
    let mut min_adaptive_radiation_gain = f64::INFINITY;
    let mut min_adaptive_channel_information_gain = f64::INFINITY;
    let mut min_scar_radiation_gain = f64::INFINITY;
    let mut min_scar_channel_information_gain = f64::INFINITY;

    push_line(
        &mut output,
        "Universum multi-seed primitive-stack falsification report",
    );
    push_line(&mut output, "schema=1");
    push_line(&mut output, "operator_disposition=none");
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
        let report = run_one(options, seed)?;
        let adaptive_radiation_gain = gain(
            report.baseline.measurements.radiated,
            report.inert.measurements.radiated,
        );
        let adaptive_channel_information_gain =
            report.baseline.measurements.channel_information_bits
                - report.inert.measurements.channel_information_bits;
        let scar_radiation_gain = gain(
            report.scar_adaptive.measurements.radiated,
            report.scar_inert.measurements.radiated,
        );
        let scar_channel_information_gain =
            report.scar_adaptive.measurements.channel_information_bits
                - report.scar_inert.measurements.channel_information_bits;
        let max_error = report
            .outcomes()
            .iter()
            .map(|outcome| outcome.relative_accounting_error)
            .fold(0.0, f64::max);
        let deterministic = report
            .outcomes()
            .iter()
            .all(|outcome| outcome.deterministic);
        let failures = report.failures();

        passed &= failures.is_empty();
        max_relative_accounting_error = max_relative_accounting_error.max(max_error);
        min_adaptive_radiation_gain = min_adaptive_radiation_gain.min(adaptive_radiation_gain);
        min_adaptive_channel_information_gain =
            min_adaptive_channel_information_gain.min(adaptive_channel_information_gain);
        min_scar_radiation_gain = min_scar_radiation_gain.min(scar_radiation_gain);
        min_scar_channel_information_gain =
            min_scar_channel_information_gain.min(scar_channel_information_gain);

        push_line(&mut output, &format!("seed_run={index}"));
        push_line(&mut output, &format!("  seed={seed}"));
        push_line(&mut output, &format!("  passed={}", failures.is_empty()));
        push_line(&mut output, &format!("  deterministic={deterministic}"));
        push_line(
            &mut output,
            &format!("  max_relative_accounting_error={max_error:.12}"),
        );
        push_line(
            &mut output,
            &format!("  adaptive_radiation_gain={adaptive_radiation_gain:.9}"),
        );
        push_line(
            &mut output,
            &format!("  adaptive_channel_information_gain={adaptive_channel_information_gain:.9}"),
        );
        push_line(
            &mut output,
            &format!("  scar_radiation_gain={scar_radiation_gain:.9}"),
        );
        push_line(
            &mut output,
            &format!("  scar_channel_information_gain={scar_channel_information_gain:.9}"),
        );
        for failure in failures {
            push_line(&mut output, &format!("  failure={failure}"));
        }
    }

    push_line(&mut output, "");
    push_line(
        &mut output,
        &format!("max_relative_accounting_error={max_relative_accounting_error:.12}"),
    );
    push_line(
        &mut output,
        &format!("min_adaptive_radiation_gain={min_adaptive_radiation_gain:.9}"),
    );
    push_line(
        &mut output,
        &format!(
            "min_adaptive_channel_information_gain={min_adaptive_channel_information_gain:.9}"
        ),
    );
    push_line(
        &mut output,
        &format!("min_scar_radiation_gain={min_scar_radiation_gain:.9}"),
    );
    push_line(
        &mut output,
        &format!("min_scar_channel_information_gain={min_scar_channel_information_gain:.9}"),
    );
    push_line(
        &mut output,
        &format!(
            "cohort_falsification_gate={}",
            if passed { "PASS" } else { "FAIL" }
        ),
    );
    Ok((output, passed))
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
            seed_count: 1,
            output: PathBuf::from("artifacts/primitive-stack-falsification.txt"),
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
        "universum-falsify - compare substrate primitive stacks\n\
         \nUSAGE:\n  universum-falsify [OPTIONS]\n\
         \nOPTIONS:\n\
         \x20 --width N       field width (default: 56)\n\
         \x20 --height N      field height (default: 36)\n\
         \x20 --moments N     moments per stack (default: 1200)\n\
         \x20 --seed N        deterministic initial grain (default: 103)\n\
         \x20 --seed-stride N increment between seeds (default: 104729)\n\
         \x20 --seed-count N  number of seeds, 1..64 (default: 1)\n\
         \x20 --output PATH   report artifact path\n\
         \x20 -h, --help      print this help"
    );
}

fn gain(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() <= f64::EPSILON {
        f64::INFINITY
    } else {
        numerator / denominator
    }
}

fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_output_path() {
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
                "x.txt",
            ]
            .map(String::from)
            .into_iter(),
        )
        .unwrap();
        assert_eq!(options.moments, 12);
        assert_eq!(options.seed, 9);
        assert_eq!(options.seed_stride, 17);
        assert_eq!(options.seed_count, 3);
        assert_eq!(options.output, PathBuf::from("x.txt"));
    }

    #[test]
    fn parser_rejects_zero_seed_count() {
        let result = Options::parse(["--seed-count", "0"].map(String::from).into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn cohort_report_records_each_seed_before_gate() {
        let options = Options {
            width: 32,
            height: 24,
            moments: 420,
            seed: 103,
            seed_stride: 17,
            seed_count: 2,
            output: PathBuf::from("unused.txt"),
        };
        let (report, passed) = run_cohort(&options).unwrap();
        assert!(passed, "{report}");
        let gate = report.find("cohort_falsification_gate=").unwrap();
        for seed_run in ["seed_run=0", "seed_run=1"] {
            let position = report
                .find(seed_run)
                .unwrap_or_else(|| panic!("missing {seed_run}"));
            assert!(position < gate);
        }
        assert!(report.contains("operator_disposition=none"));
    }
}
