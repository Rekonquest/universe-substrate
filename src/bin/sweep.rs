use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use universum_substrate::run_standard_sweep;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("universum-sweep: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1))?;
    let report = run_standard_sweep(options.width, options.height, options.seed, options.moments)?;
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
            width: 72,
            height: 44,
            moments: 1_600,
            seed: 103,
            output: PathBuf::from("artifacts/primitive-optimization-sweep.txt"),
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
        "universum-sweep - test primitive optimization candidates without operator disposition\n\
         \nUSAGE:\n  universum-sweep [OPTIONS]\n\
         \nOPTIONS:\n\
         \x20 --width N       field width (default: 72)\n\
         \x20 --height N      field height (default: 44)\n\
         \x20 --moments N     moments per candidate (default: 1600)\n\
         \x20 --seed N        deterministic initial grain (default: 103)\n\
         \x20 --output PATH   report artifact path\n\
         \x20 -h, --help      print this help"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_sweep_controls() {
        let options = Options::parse(
            [
                "--moments",
                "12",
                "--seed",
                "9",
                "--width",
                "40",
                "--height",
                "30",
                "--output",
                "sweep.txt",
            ]
            .map(String::from)
            .into_iter(),
        )
        .unwrap();
        assert_eq!(options.moments, 12);
        assert_eq!(options.seed, 9);
        assert_eq!(options.width, 40);
        assert_eq!(options.height, 30);
        assert_eq!(options.output, PathBuf::from("sweep.txt"));
    }
}
