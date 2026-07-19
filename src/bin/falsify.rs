use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use universum_substrate::{FalsificationThresholds, run_standard_falsification};

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
    let report = run_standard_falsification(
        options.width,
        options.height,
        options.seed,
        options.moments,
        FalsificationThresholds::default(),
    )?;
    if let Some(parent) = options.output.parent() {
        fs::create_dir_all(parent).map_err(io_error("create report directory"))?;
    }
    let mut file = File::create(&options.output).map_err(io_error("create report"))?;
    write!(file, "{report}").map_err(io_error("write report"))?;
    file.flush().map_err(io_error("flush report"))?;

    println!("{report}");
    println!("artifact: {}", options.output.display());
    Ok(report.passed())
}

fn io_error(context: &'static str) -> impl FnOnce(io::Error) -> String {
    move |error| format!("could not {context}: {error}")
}

struct Options {
    width: usize,
    height: usize,
    moments: u64,
    seed: u64,
    output: PathBuf,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            width: 56,
            height: 36,
            moments: 1_200,
            seed: 103,
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
                "--width" | "--height" | "--moments" | "--seed" | "--output" => arguments
                    .next()
                    .ok_or_else(|| format!("{argument} requires a value"))?,
                _ => return Err(format!("unknown argument: {argument}")),
            };
            match argument.as_str() {
                "--width" => options.width = parse_number(&argument, &value)?,
                "--height" => options.height = parse_number(&argument, &value)?,
                "--moments" => options.moments = parse_number(&argument, &value)?,
                "--seed" => options.seed = parse_number(&argument, &value)?,
                "--output" => options.output = PathBuf::from(value),
                _ => unreachable!(),
            }
        }
        if options.moments == 0 {
            return Err("--moments must be greater than zero".into());
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
         \x20 --output PATH   report artifact path\n\
         \x20 -h, --help      print this help"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_output_path() {
        let options = Options::parse(
            ["--moments", "12", "--seed", "9", "--output", "x.txt"]
                .map(String::from)
                .into_iter(),
        )
        .unwrap();
        assert_eq!(options.moments, 12);
        assert_eq!(options.seed, 9);
        assert_eq!(options.output, PathBuf::from("x.txt"));
    }
}
