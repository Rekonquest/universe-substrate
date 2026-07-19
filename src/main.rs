use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
    time::Instant,
};

use universum_substrate::{Config, CouplingMode, DisturbanceMode, Measurements, World, write_bmp};

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
        coupling_mode: options.coupling_mode,
        disturbance_mode: options.disturbance_mode,
        ..Config::default()
    };
    let mut world = World::new(config)?;
    let started = Instant::now();
    let samples = evolve_with_optional_sampling(&mut world, &options);
    let elapsed = started.elapsed();
    let measured = world.measurements();

    if let Some(parent) = options.output.parent() {
        fs::create_dir_all(parent).map_err(io_error("create artifact directory"))?;
    }
    write_bmp(&world, &options.output).map_err(io_error("write visible field"))?;

    let report_path = options.output.with_extension("txt");
    write_report(&report_path, &options, measured, elapsed.as_secs_f64())
        .map_err(io_error("write measurement report"))?;
    let timeline_path = if options.sample_every.is_some() {
        let timeline_path = options.output.with_extension("csv");
        write_timeline(&timeline_path, &samples).map_err(io_error("write timeline report"))?;
        Some(timeline_path)
    } else {
        None
    };

    println!("field:       {} x {}", world.width(), world.height());
    println!("coupling:    {}", options.coupling_mode.as_str());
    println!("disturbance: {}", options.disturbance_mode.as_str());
    println!("moments:     {}", measured.age);
    println!("elapsed:     {:.3} s", elapsed.as_secs_f64());
    println!("introduced:  {:.6}", measured.introduced);
    println!("resident:    {:.6}", measured.resident);
    println!("radiated:    {:.6}", measured.radiated);
    println!("dissipated:  {:.6}", measured.dissipated);
    println!(
        "disturbance: +{:.6} / -{:.6}",
        measured.disturbance_introduced, measured.disturbance_dissipated
    );
    println!("error:       {:.3e}", measured.accounting_error);
    println!("organized:   {:.2}%", measured.organized_fraction * 100.0);
    println!("coupled:     {:.2}%", measured.coupled_fraction * 100.0);
    println!(
        "component:   {:.2}%",
        measured.largest_organized_component * 100.0
    );
    println!(
        "channel:     {:.2}% fidelity, {:.2}% balance, {:.4} bits",
        measured.channel_fidelity * 100.0,
        measured.channel_balance * 100.0,
        measured.channel_information_bits
    );
    println!("luminous:    {} sites", measured.luminous_sites);
    println!("artifact:    {}", options.output.display());
    println!("measurements:{}", report_path.display());
    if let (Some(sample_every), Some(timeline_path)) = (options.sample_every, timeline_path) {
        println!("sampled:     every {sample_every} moments");
        println!("timeline:    {}", timeline_path.display());
    }
    Ok(())
}

fn evolve_with_optional_sampling(world: &mut World, options: &Options) -> Vec<Measurements> {
    let Some(sample_every) = options.sample_every else {
        world.evolve_for(options.moments);
        return Vec::new();
    };

    let mut samples = Vec::new();
    samples.push(world.measurements());
    let mut remaining = options.moments;
    while remaining > 0 {
        let step = remaining.min(sample_every);
        world.evolve_for(step);
        samples.push(world.measurements());
        remaining -= step;
    }
    samples
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
    writeln!(report, "coupling={}", options.coupling_mode.as_str())?;
    writeln!(report, "disturbance={}", options.disturbance_mode.as_str())?;
    writeln!(report, "width={}", options.width)?;
    writeln!(report, "height={}", options.height)?;
    writeln!(report, "moments={}", measured.age)?;
    if let Some(sample_every) = options.sample_every {
        writeln!(report, "sample_every={sample_every}")?;
        writeln!(
            report,
            "timeline={}",
            options.output.with_extension("csv").display()
        )?;
    }
    writeln!(report, "elapsed_seconds={elapsed_seconds:.6}")?;
    writeln!(report, "introduced={:.9}", measured.introduced)?;
    writeln!(report, "resident={:.9}", measured.resident)?;
    writeln!(report, "radiated={:.9}", measured.radiated)?;
    writeln!(report, "dissipated={:.9}", measured.dissipated)?;
    writeln!(
        report,
        "disturbance_introduced={:.9}",
        measured.disturbance_introduced
    )?;
    writeln!(
        report,
        "disturbance_dissipated={:.9}",
        measured.disturbance_dissipated
    )?;
    writeln!(report, "accounting_error={:.12}", measured.accounting_error)?;
    writeln!(
        report,
        "mean_permeability={:.9}",
        measured.mean_permeability
    )?;
    writeln!(report, "mean_coupling={:.9}", measured.mean_coupling)?;
    writeln!(
        report,
        "organized_fraction={:.9}",
        measured.organized_fraction
    )?;
    writeln!(report, "coupled_fraction={:.9}", measured.coupled_fraction)?;
    writeln!(
        report,
        "largest_organized_component={:.9}",
        measured.largest_organized_component
    )?;
    writeln!(report, "luminous_sites={}", measured.luminous_sites)?;
    writeln!(report, "channel_signal={:.9}", measured.channel_signal)?;
    writeln!(report, "channel_total={:.9}", measured.channel_total)?;
    writeln!(report, "channel_fidelity={:.9}", measured.channel_fidelity)?;
    writeln!(report, "channel_balance={:.9}", measured.channel_balance)?;
    writeln!(
        report,
        "channel_information_bits={:.9}",
        measured.channel_information_bits
    )?;
    Ok(())
}

fn write_timeline(path: &PathBuf, samples: &[Measurements]) -> io::Result<()> {
    let mut report = File::create(path)?;
    writeln!(
        report,
        "age,introduced,resident,radiated,dissipated,accounting_error,disturbance_introduced,disturbance_dissipated,mean_permeability,mean_coupling,organized_fraction,coupled_fraction,largest_organized_component,luminous_sites,channel_signal,channel_total,channel_fidelity,channel_balance,channel_information_bits"
    )?;
    for measured in samples {
        writeln!(
            report,
            "{},{:.9},{:.9},{:.9},{:.9},{:.12},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{},{:.9},{:.9},{:.9},{:.9},{:.9}",
            measured.age,
            measured.introduced,
            measured.resident,
            measured.radiated,
            measured.dissipated,
            measured.accounting_error,
            measured.disturbance_introduced,
            measured.disturbance_dissipated,
            measured.mean_permeability,
            measured.mean_coupling,
            measured.organized_fraction,
            measured.coupled_fraction,
            measured.largest_organized_component,
            measured.luminous_sites,
            measured.channel_signal,
            measured.channel_total,
            measured.channel_fidelity,
            measured.channel_balance,
            measured.channel_information_bits
        )?;
    }
    Ok(())
}

struct Options {
    width: usize,
    height: usize,
    moments: u64,
    seed: u64,
    coupling_mode: CouplingMode,
    disturbance_mode: DisturbanceMode,
    sample_every: Option<u64>,
    output: PathBuf,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            width: 192,
            height: 112,
            moments: 4_000,
            seed: 0xA701_5EED,
            coupling_mode: CouplingMode::Adaptive,
            disturbance_mode: DisturbanceMode::None,
            sample_every: None,
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
                "--width" | "--height" | "--moments" | "--seed" | "--coupling"
                | "--disturbance" | "--sample-every" | "--output" => arguments
                    .next()
                    .ok_or_else(|| format!("{argument} requires a value"))?,
                _ => return Err(format!("unknown argument: {argument}")),
            };
            match argument.as_str() {
                "--width" => options.width = parse_number(&argument, &value)?,
                "--height" => options.height = parse_number(&argument, &value)?,
                "--moments" => options.moments = parse_number(&argument, &value)?,
                "--seed" => options.seed = parse_number(&argument, &value)?,
                "--coupling" => options.coupling_mode = parse_coupling(&value)?,
                "--disturbance" => options.disturbance_mode = parse_disturbance(&value)?,
                "--sample-every" => options.sample_every = Some(parse_number(&argument, &value)?),
                "--output" => options.output = PathBuf::from(value),
                _ => unreachable!(),
            }
        }
        if options.moments == 0 {
            return Err("--moments must be greater than zero".into());
        }
        if options.sample_every == Some(0) {
            return Err("--sample-every must be greater than zero".into());
        }
        Ok(options)
    }
}

fn parse_coupling(value: &str) -> Result<CouplingMode, String> {
    match value {
        "adaptive" => Ok(CouplingMode::Adaptive),
        "fixed" => Ok(CouplingMode::Fixed),
        "inert" => Ok(CouplingMode::Inert),
        _ => Err(format!(
            "invalid value for --coupling: {value}; use adaptive, fixed, or inert"
        )),
    }
}

fn parse_disturbance(value: &str) -> Result<DisturbanceMode, String> {
    match value {
        "none" => Ok(DisturbanceMode::None),
        "scar" => Ok(DisturbanceMode::Scar),
        "noise" => Ok(DisturbanceMode::Noise),
        _ => Err(format!(
            "invalid value for --disturbance: {value}; use none, scar, or noise"
        )),
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
         \x20 --moments N     evolution moments (default: 4000)\n\
         \x20 --seed N        deterministic initial grain\n\
         \x20 --coupling MODE adaptive, fixed, or inert (default: adaptive)\n\
         \x20 --disturbance MODE none, scar, or noise (default: none)\n\
         \x20 --sample-every N write a CSV timeline every N moments\n\
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
                "--coupling",
                "fixed",
                "--disturbance",
                "scar",
                "--sample-every",
                "4",
            ]
            .map(String::from)
            .into_iter(),
        )
        .unwrap();
        assert_eq!(options.moments, 12);
        assert_eq!(options.seed, 9);
        assert_eq!(options.width, 32);
        assert_eq!(options.height, 24);
        assert_eq!(options.coupling_mode, CouplingMode::Fixed);
        assert_eq!(options.disturbance_mode, DisturbanceMode::Scar);
        assert_eq!(options.sample_every, Some(4));
    }

    #[test]
    fn parser_rejects_unknown_coupling_mode() {
        let result = Options::parse(["--coupling", "queue"].map(String::from).into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn parser_rejects_unknown_disturbance_mode() {
        let result = Options::parse(["--disturbance", "panic"].map(String::from).into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn parser_rejects_zero_sample_interval() {
        let result = Options::parse(["--sample-every", "0"].map(String::from).into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn sampling_records_initial_intermediate_and_final_measurements() {
        let options = Options {
            width: 32,
            height: 24,
            moments: 10,
            seed: 5,
            coupling_mode: CouplingMode::Adaptive,
            disturbance_mode: DisturbanceMode::None,
            sample_every: Some(4),
            output: PathBuf::from("unused.bmp"),
        };
        let config = Config {
            width: options.width,
            height: options.height,
            seed: options.seed,
            coupling_mode: options.coupling_mode,
            disturbance_mode: options.disturbance_mode,
            ..Config::default()
        };
        let mut world = World::new(config).unwrap();
        let samples = evolve_with_optional_sampling(&mut world, &options);
        let ages = samples.iter().map(|sample| sample.age).collect::<Vec<_>>();
        assert_eq!(ages, vec![0, 4, 8, 10]);
    }
}
