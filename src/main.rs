use std::path::PathBuf;
use hyperrealism_core::pipeline::Options;
fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut input = PathBuf::from("work/material_manifest.json");
    let mut output = PathBuf::from(".");
    let mut threads = None;
    let mut options = Options::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = PathBuf::from(args.next().ok_or("--input requires a path")?),
            "--output" => output = PathBuf::from(args.next().ok_or("--output requires a path")?),
            "--cpu-only" => (),
            "--resolution" | "--seed" | "--name" => {
                let text = args.next().ok_or(format!("{arg} requires a value"))?;
                match arg.as_str() {
                    "--resolution" => options.resolution = text.parse().map_err(|_| "resolution must be 64, 128, or 256")?,
                    "--seed" => options.seed = text.parse().map_err(|_| "seed must be an unsigned 32-bit integer")?,
                    _ => options.name = text,
                }
            }
            "--no-normal" => options.include_normal = false,
            "--no-specular" => options.include_specular = false,
            "--threads" => {
                let text = args.next().ok_or("--threads requires a positive integer")?;
                let count: usize = text
                    .parse()
                    .map_err(|_| "--threads requires a positive integer")?;
                if count == 0 || count > 256 {
                    return Err("--threads must be in 1..=256".into());
                }
                threads = Some(count);
            }
            "--help" | "-h" => {
                println!("HyperRealism CPU resource-pack generator\n\nUsage: hyperrealism-core [--input MANIFEST_OR_DIR] [--output DIR] [--cpu-only] [--threads N] [--resolution 64|128|256] [--seed N] [--name NAME] [--no-normal] [--no-specular]\n\nThe input directory must contain material_manifest.json. CPU execution only. Existing output is protected; use a fresh output directory.");
                return Ok(());
            }
            _ => return Err(format!("unknown option: {arg}; use --help")),
        }
    }
    if let Some(count) = threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(count)
            .build_global()
            .map_err(|e| format!("could not create thread pool: {e}"))?;
    }
    println!(
        "Backend: CPU (GPU acceleration not implemented); Rayon threads: {}",
        rayon::current_num_threads()
    );
    hyperrealism_core::pipeline::generate_with_options(&input, &output, &options)
}
