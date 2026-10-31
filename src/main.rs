use std::path::PathBuf;
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
    let mut minecraft_jar = None;
    let mut verify_pack = None;
    let mut size = 512usize;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = PathBuf::from(args.next().ok_or("--input requires a path")?),
            "--output" => output = PathBuf::from(args.next().ok_or("--output requires a path")?),
            "--cpu-only" => (),
            "--verify-pack" => {
                verify_pack = Some(PathBuf::from(
                    args.next()
                        .ok_or("--verify-pack requires a pack directory")?,
                ))
            }
            "--minecraft-jar" => {
                minecraft_jar = Some(PathBuf::from(
                    args.next().ok_or("--minecraft-jar requires a path")?,
                ))
            }
            "--resolution" => {
                let value = args.next().ok_or("--resolution requires 128 or 512")?;
                size = value
                    .parse()
                    .map_err(|_| "--resolution requires 128 or 512")?;
                if size != 128 && size != 512 {
                    return Err("--resolution requires 128 or 512".into());
                }
            }
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
                println!("HyperRealism CPU prototype\n\nUsage: hyperrealism-core [--input MANIFEST_OR_DIR] [--output DIR] [--cpu-only] [--threads N] [--resolution 128|512] [--minecraft-jar CLIENT_JAR] [--verify-pack PACK_DIR]\n\nThe input directory must contain material_manifest.json. This prototype supports CPU execution only and does not yet generate the full catalog.");
                return Ok(());
            }
            _ => return Err(format!("unknown option: {arg}; use --help")),
        }
    }
    if let Some(path) = verify_pack {
        let (block_count, entity_count) = hyperrealism_core::verify::full_pack(&path)?;
        println!(
            "Verified {block_count} block and {entity_count} entity LabPBR texture triples in {} (visual/shader testing still required)",
            path.display()
        );
        return Ok(());
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
    hyperrealism_core::pipeline::generate(&input, &output, minecraft_jar.as_deref(), size)
}
