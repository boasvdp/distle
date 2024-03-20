use std::error::Error;
use std::io::BufReader;
use std::time::Instant;

use clap::Parser;
use env_logger::Env;
use log::debug;

mod processing;
mod types;

use log::info;
use processing::{
    compute_distances, read_and_parse_fasta, read_and_parse_tabular, write_distances_to_file,
    OutputFormat, OutputMode,
};
use types::InputFormat;

/// This struct represents the command-line arguments
#[derive(Parser, Debug)]
struct Cli {
    input: String,
    output: String,
    #[arg(value_enum, short = 'i', long, default_value = "fasta")]
    input_format: InputFormat,

    #[arg(value_enum, short = 'o', long, default_value = "tabular")]
    output_format: OutputFormat,

    #[arg(long, default_value = "\t")]
    input_sep: char,

    #[arg(long, default_value = "\t")]
    output_sep: char,

    #[arg(value_enum, short = 'm', long, default_value = "lower-triangle")]
    output_mode: OutputMode,

    #[arg(short = 'd', long, default_value = None)]
    maxdist: Option<usize>,

    #[arg(short = 's', long)]
    skip_header: bool,

    #[arg(short = 'v', long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Cli = Cli::parse();
    if opts.verbose {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    }

    let reader: Box<dyn std::io::Read> = if opts.input == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(std::fs::File::open(&opts.input)?)
    };

    let reader = BufReader::new(reader);

    // print version info
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // print command line arguments
    debug!("Cli options: {:?}", opts);

    let start = Instant::now();
    let data_map = match opts.input_format {
        InputFormat::Fasta | InputFormat::FastaAll => {
            read_and_parse_fasta(reader, opts.input_format)?
        }
        _ => read_and_parse_tabular(reader, opts.input_format, opts.input_sep, opts.skip_header)?,
    };
    debug!("Reading time: {:?}", start.elapsed());
    let start = Instant::now();

    info!("Computing distances and writing to file: {}", &opts.output);

    // Compute the pairwise distances
    let distances = compute_distances(&data_map, opts.maxdist, opts.output_mode);

    let writer: Box<dyn std::io::Write> = if opts.output == "-" {
        Box::new(std::io::stdout())
    } else {
        Box::new(std::fs::File::create(&opts.output)?)
    };

    let mut writer = std::io::BufWriter::new(writer);

    write_distances_to_file(
        distances,
        &mut writer,
        opts.output_sep,
        opts.output_format,
        data_map.len(),
    )?;

    debug!("Computing + Writing time: {:?}", start.elapsed());
    match opts.maxdist {
        Some(maxdist) => info!("Computed distances with a maximum distance of {}", maxdist),
        None => info!("Computed all distances"),
    }

    info!("Done");

    Ok(())
}
