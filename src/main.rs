use clap::{Arg, Command};
use std::{
    fs::{read_to_string, File},
    io::Write,
    path::Path,
    process::exit,
};
use yara::Compiler;
use yara_dedupe::{
    nom::parse_vec,
    utils::{collect_yar_files, remove_comments},
};

fn main() {
    let matches = Command::new("Yara Dedupe")
        .about("Dedupe & Compile Yara Rules")
        .subcommand(
            Command::new("dedupe")
                .about("Dedupe given yara rules")
                .arg(
                    Arg::new("input_dir")
                        .help("directory containing the yara rule files for dedupe")
                        .short('i')
                        .long("input-dir")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::new("output_file")
                        .help("output file name to store the deduplicated single yara file")
                        .short('o')
                        .long("output-file")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            Command::new("compile")
                .about("Compiles a given Yara ruleset")
                .arg(
                    Arg::new("input_file")
                        .help("yara ruleset file to compile")
                        .required(true),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("dedupe", dedupe_args)) => {
            let input_dir = dedupe_args.value_of("input_dir").unwrap();
            let input_dir = if input_dir.is_empty() {
                println!("Input folder not define: {:?}", input_dir);
                exit(1)
            } else if !Path::new(input_dir).is_dir() {
                println!("Input folder not dir or not exists: {:?}", input_dir);
                exit(1)
            } else {
                input_dir
            };
            let output_file = dedupe_args.value_of("output_file").unwrap();

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
        }
        Some(("compile", compile_args)) => {
            let input_file = compile_args.value_of("input_file").unwrap();
            let compiler = Compiler::new()
                .unwrap()
                .add_rules_file(input_file)
                .unwrap();
            let compiled_output_file = format!("compiled_{}", input_file);
            let mut rules = compiler
                .compile_rules()
                .expect("Couldn't compile the rules");
            rules
                .save(&compiled_output_file)
                .expect("Couldn't save the compiled rules");

            println!(
                "* Compiled yara ruleset is stored in: {}",
                compiled_output_file
            );
        }
        None => println!("No command passed. Nothing to do."),
        _ => unreachable!(),
    }

    println!();
}
