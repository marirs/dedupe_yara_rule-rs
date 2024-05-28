use filepath::FilePath;
use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::{Read, Write},
    path::Path,
    process::exit,
};
use yara_dedupe::{
    cli::{CliOpts, Compile, Dedupe, SubCommand},
    nom::parse_rules,
    utils::collect_yar_files,
};
use yara_x::Compiler;

fn main() {
    let configuration = CliOpts::parse_cli();
    match configuration.cmd {
        SubCommand::Dedupe(dedupe) => {
            dedupe_rules(dedupe);
        }
        SubCommand::Compile(compile) => {
            compile_rules(compile);
        }
    }
    println!();
}

/// Extracts rule information from input directories, generates a deduplicated set of YARA rules, and writes them to an output file.
///
/// # Arguments
///
/// * `dedupe` - A `Dedupe` struct containing the configuration for deduplication.
///
/// # Panics
///
/// This function will panic if any file or directory paths specified in `dedupe` are invalid or if there is an error creating the output file.
///
/// # Examples
///
/// ```
/// let dedupe = Dedupe {
///     input_dir: vec!["input_dir"],
///     output_file: "output_file.yar",
///     skip_rules: Some("skip_rules.yar"),
/// };
/// dedupe_rules(dedupe);
/// ```
fn dedupe_rules(dedupe: Dedupe) {
    let input_dirs: Vec<&str> = dedupe.input_dir.iter().map(|x| x.as_str()).collect();
    let output_file = dedupe.output_file;
    let skip_rules = if let Some(f) = dedupe.skip_rules {
        if !Path::new(&f).is_file() {
            println!("Skip Rules File with Rule names does not exist: {:?}", f);
            exit(1)
        }
        let contents = read_to_string(f).unwrap();
        contents.split('\n').map(|x| x.to_string()).collect()
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

    let mut f = File::create(output_file.clone()).expect("error creating output yara file");
    for i in &all_yars.imports {
        writeln!(f, "import {}", i).expect("error in writing \"imports\" to output file")
    }
    writeln!(f).expect("error in writing to file");
    write!(f, "{}", all_yars).expect("error in writing \"yara rules\" to output file");
    println!("* Output yara file stored in: {}", output_file);
}

/// Compiles YARA rules given a `Compile` configuration.
///
/// # Arguments
///
/// * `compile` - A `Compile` configuration object that specifies the input file path.
///
/// # Panics
///
/// This function will panic if any of the following conditions occur:
///
/// * The input file does not exist.
/// * An error occurred while reading the input file.
/// * An error occurred while adding the rule to the compiler.
/// * An error occurred while serializing the compiled rules.
/// * An error occurred while creating the compiled output file.
/// * An error occurred while writing the compiled rules to the file.
///
/// # Examples
///
/// ```
/// let compile = Compile { input_file: "rules.yara" };
/// compile_rules(compile);
/// ```
fn compile_rules(compile: Compile) {
    let input_file = Path::new(&compile.input_file);
    if !input_file.is_file() {
        println!("Input file not found: {:?}", input_file);
        exit(1)
    }
    let file_content = read_to_string(input_file).unwrap();
    let mut compiler = Compiler::new();
    if let Err(e) = compiler.add_source(file_content.as_str()) {
        match e {
            yara_x::Error::CompileError(_) => print_error("COMPILE error", &e),
            yara_x::Error::ParseError(_) => print_error("PARSE error", &e),
            yara_x::Error::VariableError(_) => print_error("INVALID VARIABLE", &e),
        };
        exit(1);
    };
    println!("* Rules added to the compiler successfully");

    let compiled_output_file = format!(
        "compiled_{}",
        input_file.file_name().unwrap().to_str().unwrap()
    );
    let rules = compiler.build();
    let serialized_rules = rules.serialize().unwrap();
    let mut compiled_output_file = if let Ok(f) = File::create(compiled_output_file.clone()) {
        f
    } else {
        println!(
            "Couldn't create the compiled output file: {}",
            compiled_output_file
        );
        exit(1)
    };
    let output_fn = compiled_output_file.path().unwrap().display().to_string();
    compiled_output_file
        .write_all(&serialized_rules)
        .unwrap_or_else(|_| {
            panic!(
                "Couldn't write the compiled rules to the file: {}",
                output_fn
            )
        });

    println!("* Compiled yara ruleset is stored in: {}", output_fn);
}

#[inline]
fn print_error(error_msg: &str, error: &yara_x::Error) {
    println!("Couldn't add rule; {}: {:#?}", error_msg, error);
}
