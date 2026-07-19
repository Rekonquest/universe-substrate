use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
    time::Instant,
};

use universum_substrate::{Config, Measurements, World, write_bmp};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("universum: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1))?;
    let config = Config {
        width: options.width,
        height: options.height,
        seed: options.seed,
        ..Config::default()
    };
    let mut world = World::new(config)?;
    let started = Instant::now();
    world.evolve_for(options.moments);
    let elapsed = started.elapsed();
    let measured = world.measurements();

    if let Some(parent) = options.output.parent() {
        fs::create_dir_all(parent).map_err(io_error("create artifact directory"))?;
    }
    write_bmp(&world, &options.output).map_err(io_error("write visible field"))?;

    let report_path = options.output.with_extension("txt");
    write_report(&report_path, &options, measured, elapsed.as_secs_f64())
        .map_err(io_error("write measurement report"))?;

    println!("field:       {} x {}", world.width(), world.height());
    println!("moments:     {}", measured.age);
    println!("elapsed:     {:.3} s", elapsed.as_secs_f64());
    println!("introduced:  {:.6}", measured.introduced);
    println!("resident:    {:.6}", measured.resident);
    println!("radiated:    {:.6}", measured.radiated);
    println!("dissipated:  {:.6}", measured.dissipated);
    println!("error:       {:.3e}", measured.accounting_error);
    println!("organized:   {:.2}%", measured.organized_fraction * 100.0);
    println!("luminous:    {} sites", measured.luminous_sites);
    println!("artifact:    {}", options.output.display());
    println!("measurements:{}", report_path.display());
    Ok(())
}

fn io_error(context: &'static str) -> impl FnOnce(io::Error) -> String {
    move |error| format!("could not {context}: {error}")
}

fn write_report(
    path: &PathBuf,
    options: &Options,
    measured: Measurements,
    elapsed_seconds: f64,
) -> io::Result<()> {
    let mut report = File::create(path)?;
    writeln!(report, "Universum substrate experiment")?;
    writeln!(report, "seed={}", options.seed)?;
    writeln!(report, "width={}", options.width)?;
    writeln!(report, "height={}", options.height)?;
    writeln!(report, "moments={}", measured.age)?;
    writeln!(report, "elapsed_seconds={elapsed_seconds:.6}")?;
    writeln!(report, "introduced={:.9}", measured.introduced)?;
    writeln!(report, "resident={:.9}", measured.resident)?;
    writeln!(report, "radiated={:.9}", measured.radiated)?;
    writeln!(report, "dissipated={:.9}", measured.dissipated)?;
    writeln!(report, "accounting_error={:.12}", measured.accounting_error)?;
    writeln!(
        report,
        "mean_permeability={:.9}",
        measured.mean_permeability
    )?;
    writeln!(
        report,
        "organized_fraction={:.9}",
        measured.organized_fraction
    )?;
    writeln!(report, "luminous_sites={}", measured.luminous_sites)?;
    Ok(())
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
            width: 192,
            height: 112,
            moments: 2_400,
            seed: 0xA701_5EED,
            output: PathBuf::from("artifacts/universe-frame.bmp"),
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
        "universum - evolve a visible field from local physical laws\n\
         \nUSAGE:\n  universum [OPTIONS]\n\
         \nOPTIONS:\n\
         \x20 --width N       field width (default: 192)\n\
         \x20 --height N      field height (default: 112)\n\
         \x20 --moments N     evolution moments (default: 2400)\n\
         \x20 --seed N        deterministic initial grain\n\
         \x20 --output PATH   bitmap artifact path\n\
         \x20 -h, --help      print this help"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_rejects_unknown_arguments() {
        let result = Options::parse(["--human-driver".to_string()].into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn parser_accepts_experiment_controls() {
        let options = Options::parse(
            [
                "--moments",
                "12",
                "--seed",
                "9",
                "--width",
                "32",
                "--height",
                "24",
            ]
            .map(String::from)
            .into_iter(),
        )
        .unwrap();
        assert_eq!(options.moments, 12);
        assert_eq!(options.seed, 9);
        assert_eq!(options.width, 32);
        assert_eq!(options.height, 24);
    }
}
