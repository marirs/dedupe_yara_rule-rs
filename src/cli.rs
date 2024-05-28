use clap::{Args, Parser, Subcommand};

/// Dedupe & Compile Yara Rules
#[derive(Parser)]
#[clap(name = "Yara Dedupe")]
#[command(version, about)]
pub struct CliOpts {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    /// Dedupe given yara rules
    Dedupe(Dedupe),
    /// Compiles a given Yara ruleset
    Compile(Compile),
}

/// A subcommand for deduping yara rules
#[derive(Args)]
pub struct Dedupe {
    /// directory containing the yara rule files for dedupe
    #[clap(short = 'i', long = "input-dir", required = true)]
    pub input_dir: Vec<String>,

    /// output file name to store the deduplicated single yara file
    #[clap(short = 'o', long = "output-file", required = true)]
    pub output_file: String,

    /// skips a list of rules specified in a file
    #[clap(long = "skip-rules")]
    pub skip_rules: Option<String>,
}

/// A subcommand for compiling yara rules
#[derive(Args)]
pub struct Compile {
    /// yara ruleset file to compile
    #[clap(required = true)]
    pub input_file: String,
}

impl CliOpts {
    pub fn parse_cli() -> Self {
        Self::parse()
    }
}
