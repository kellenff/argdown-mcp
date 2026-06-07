//! `argdown` CLI: reads Argdown from stdin, writes results to stdout.
//! Diagnostics go to stderr; a non-zero exit code signals failure.

use std::io::{self, Read, Write};
use std::process::ExitCode;

use argdown_tools::{Diagnostic, Format, ToolError, dung, model_export, summarize};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "argdown",
    version,
    about = "Argdown toolchain: parse / export / dung over stdin -> stdout"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Parse Argdown from stdin; print a syntactic summary as JSON.
    Parse,
    /// Build the Layer B model from stdin and export it as JSON (default) or YAML.
    Export {
        /// Output format.
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Compute the grounded extension (IN/OUT/UNDEC) from stdin as JSON.
    Dung,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
    Yaml,
}

impl From<OutputFormat> for Format {
    fn from(f: OutputFormat) -> Self {
        match f {
            OutputFormat::Json => Format::Json,
            OutputFormat::Yaml => Format::Yaml,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(out) => {
            let mut stdout = io::stdout();
            let _ = writeln!(stdout, "{out}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("argdown: {msg}");
            ExitCode::from(1)
        }
    }
}

/// Format a parse diagnostic with its byte offset for stderr.
fn format_diagnostic(d: &Diagnostic) -> String {
    format!("{} (at byte {})", d.message, d.offset)
}

/// Dispatch a subcommand to its pure `argdown_tools` function, returning the
/// stdout payload on success or a human-readable diagnostic on failure.
fn run(cli: Cli) -> Result<String, String> {
    let source = read_stdin().map_err(|e| format!("failed to read stdin: {e}"))?;
    match cli.command {
        Command::Parse => {
            let result = summarize(&source);
            match result.summary {
                Some(summary) => serde_json::to_string_pretty(&summary).map_err(|e| e.to_string()),
                None => {
                    let d = result
                        .diagnostic
                        .expect("diagnostic present when summary absent");
                    Err(format_diagnostic(&d))
                }
            }
        }
        Command::Export { format } => model_export(&source, format.into()).map_err(|e| match e {
            ToolError::Parse(d) => format_diagnostic(&d),
            ToolError::Serialize(msg) => msg,
        }),
        Command::Dung => match dung(&source) {
            Ok(result) => serde_json::to_string_pretty(&result).map_err(|e| e.to_string()),
            Err(d) => Err(format_diagnostic(&d)),
        },
    }
}

fn read_stdin() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}
