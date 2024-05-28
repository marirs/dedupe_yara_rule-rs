use clap::{Arg, Command};
use std::{
    collections::HashMap,
    fs::{File, read_to_string},
    io::{Read, Write},
    path::Path,
    process::exit,
};
use yara_x::Compiler;
use yara_dedupe::{
    nom::parse_rules,
    utils::collect_yar_files,
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
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("output_file")
                        .help("output file name to store the deduplicated single yara file")
                        .short('o')
                        .long("output-file")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::new("skip_rules")
                        .help("skips a list of rules specified in a file")
                        .long("skip-rules")
                        .takes_value(true)
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
            let input_dirs: Vec<&str> = dedupe_args.values_of("input_dir").unwrap().collect();
            let output_file = dedupe_args.value_of("output_file").unwrap();
            let skip_rules = if let Some(f) = dedupe_args.value_of("skip_rules"){
                if !Path::new(f).is_file() {
                    println!("Skip Rules File with Rule names does not exist: {:?}", f);
                    exit(1)
                }
                let contents = read_to_string(f)
                    .unwrap();
                contents
                    .split('\n')
                    .into_iter()
                    .map(|x|x.to_string())
                    .collect()
            } else {
                vec![]
            };
            let mut all_yar_files = vec![];
            for input_dir in input_dirs {
                if !Path::new(input_dir).is_dir() {
                    println!("Input folder not dir or not exists: {:?}", input_dir);
                    exit(1)
                }

                all_yar_files.push(collect_yar_files(&input_dir))

            }

            let mut file_count = 0;

            let all_yars: HashMap<_, _> = all_yar_files
                .into_iter()
                .flatten()
                .inspect(|x| {
                    print!("\r[* examining: {:120}]", x);
                    let _ = std::io::stdout().flush();
                    file_count += 1;
                })
                .map(|path| {
                    let mut file = File::open(&path).unwrap();
                    let mut buf = vec![];
                    file.read_to_end(&mut buf).unwrap();
                    (path, String::from_utf8_lossy(&buf).to_string())
                })
                .flat_map(|(p, x)| {
                    parse_rules(
                        Path::new(&p)
                            .canonicalize()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string(),
                        &x,
                    )
                    .map(|x| (x.1.name.to_owned(), x.1))
                    .ok()
                })
                .collect();
            println!();
            let all_yars = yara_dedupe::YarAll::new(all_yars, skip_rules);
            println!("* Total files processed: {}", file_count);

            let mut f = File::create(output_file).expect("error creating output yara file");
            for i in &all_yars.imports {
                writeln!(f, "import {}", i).expect("error in writing \"imports\" to output file")
            }
            writeln!(f).expect("error in writing to file");
            write!(f, "{}", all_yars).expect("error in writing \"yara rules\" to output file");
            println!("* Output yara file stored in: {}", output_file);
        }
        Some(("compile", compile_args)) => {
            let input_file = compile_args.value_of("input_file").unwrap();
            let file_content = read_to_string(input_file).unwrap();
            let mut compiler = Compiler::new();
            if let Err(e) = compiler.add_source(file_content.as_str()) {
                if let yara_x::Error::CompileError(e) = e {
                    // for e in e. {
                    //     if e.level == yara::errors::CompileErrorLevel::Error {
                    //                                if !e.message.contains("regular expression") && !e.message.contains("unreferenced"){
                    eprintln!("Couldn't add rule: {:#?}", e);
                    //                                }
                    //     }
                    // }
                }
                panic!("");
            };

            let compiled_output_file = format!("compiled_{}", input_file);
            let rules = compiler.build();
            let serialized_rules = rules.serialize().unwrap();
            let mut compiled_output_file = File::create(&compiled_output_file).unwrap();
            compiled_output_file
                .write_all(&serialized_rules)
                .expect("Couldn't write the compiled rules to the file");

            println!(
                "* Compiled yara ruleset is stored in: {}",
                compiled_output_file.path().display()
            );
        }
        None => println!("No command passed. Nothing to do."),
        _ => unreachable!(),
    }

    println!();
}
