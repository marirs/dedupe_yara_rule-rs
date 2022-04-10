use clap::{Arg, Command};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::Path,
    process::exit,
};
use yara::Compiler;
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

            let all_yars: HashMap<_, _> = collect_yar_files(&input_dir)
                .into_iter()
                .inspect(|x| {
                    print!("\r[* examining: {:120}]", x);
                    let _ = std::io::stdout().flush();
                    file_count += 1;
                })
                .map(|path| {
                    let mut file = File::open(path.to_owned()).unwrap();
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
            let all_yars = yara_dedupe::YarAll::new(all_yars);
            println!("* Total files processed: {}", file_count);

            let mut f = File::create(output_file).expect("error creating output yara file");
            for i in &all_yars.imports {
                writeln!(f, "import {}", i).expect("error in writing \"imports\" to output file")
            }
            writeln!(f).expect("error in writing to file");
            writeln!(f, "{}\n", all_yars).expect("error in writing \"yara rules\" to output file");
            println!("* Output yara file stored in: {}", output_file);
        }
        Some(("compile", compile_args)) => {
            let input_file = compile_args.value_of("input_file").unwrap();
            let compiler = Compiler::new().unwrap().add_rules_file(input_file);
            let compiler = match compiler {
                Ok(r) => r,
                Err(e) => {
                    if let yara::Error::Compile(e) = e {
                        for e in e.iter() {
                            if e.level == yara::errors::CompileErrorLevel::Error {
                                //                                if !e.message.contains("regular expression") && !e.message.contains("unreferenced"){
                                eprintln!("Couldn't add rule: {:#?}", e);
                                //                                }
                            }
                        }
                    }
                    panic!("");
                }
            };

            let compiled_output_file = format!("compiled_{}", input_file);
            let rules = compiler.compile_rules();
            let mut rules = match rules {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Couldn't compile the rules: {:#?}", e);
                    panic!("");
                }
            };
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
