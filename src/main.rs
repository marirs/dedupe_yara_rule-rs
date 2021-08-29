use clap::{crate_authors, crate_version, Clap};
use std::{
    fs::{read_to_string, File},
    io::Write,
    path::Path,
    process::exit,
};
use yara_dedup::nom::parse_vec;
use yara_dedup::utils::{collect_yar_files, remove_comments};
use yara::Compiler;


#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct CliOpts {
    #[clap(
        short = 'i',
        long,
        about = "directory containing the yara rule files for dedupe"
    )]
    input_dir: String,
    #[clap(
        short = 'o',
        long,
        about = "output file name to store the deduplicated single yara file"
    )]
    output_file: String,
    #[clap(
        short = 'c',
        long,
        about = "enabling this will give a single deduplicated compiled yara file"
    )]
    compile: bool,
}

fn main() {
    let cli_opts = CliOpts::parse();
    let input_dir = if cli_opts.input_dir.is_empty() {
        println!("Input folder not define: {:?}", &cli_opts.input_dir);
        exit(1)
    } else if !Path::new(&cli_opts.input_dir).is_dir() {
        println!(
            "Input folder not dir or not exists: {:?}",
            &cli_opts.input_dir
        );
        exit(1)
    } else {
        &cli_opts.input_dir
    };
    let output_file = &cli_opts.output_file;
    let is_compile = cli_opts.compile;
    let mut file_count = 0;

    let mut all_yars: Vec<_> = collect_yar_files(&input_dir)
        .into_iter()
        .inspect(|x| {
            print!("\r[* examining: {:120}]", x);
            let _ = std::io::stdout().flush();
            file_count += 1;
        })
        .map(|path| read_to_string(&path).unwrap())
        .map(remove_comments)
        .flat_map(|x| parse_vec(&x).map(|x| x.1).ok())
        .flatten()
        .collect();

    println!();
    println!("* Total files processed: {}", file_count);
    println!("* Total yara rules: {}", all_yars.len());

    all_yars.sort_by_key(|x| x.name.clone());
    all_yars.dedup_by_key(|x| x.name.clone());

    println!("* Total yara rules after dedupe: {}", all_yars.len());

    File::create(output_file)
        .map(|mut f| {
            for e in all_yars {
                write!(f, "{}\n\n", e.to_string()).expect("error")
            }
        })
        .expect("error");

    println!("* Output yara file stored in: {}", output_file);

    if is_compile {
        let mut compiler = Compiler::new().unwrap();
        compiler.add_rules_file(output_file).unwrap();
        let compiled_output_file = format!("compiled_{}", output_file);
        let mut rules = compiler.compile_rules().expect("Couldn't compile the rules");
        rules.save(&compiled_output_file).expect("Couldn't save the compiled rules");

        println!("* Compiled yara ruleset is stored in: {}", compiled_output_file);
    }

    println!();
}
