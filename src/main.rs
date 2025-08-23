mod runner;
mod diagnostics; 
mod logging;
mod term;
mod util;

use clap::{Arg, ArgAction, Command};
use anyhow::Result;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub log_path: Option<String>,
    pub log_on_success: bool,
    pub log_color: ColorChoice,
    pub terminal_color: ColorChoice,
    pub include_warnings: bool,
    pub show_build_output: bool,
    pub quiet: bool,
    pub cargo_args: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ColorChoice {
    Auto,
    Never,
    Always,
}

impl std::str::FromStr for ColorChoice {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "auto" => Ok(ColorChoice::Auto),
            "never" => Ok(ColorChoice::Never),
            "always" => Ok(ColorChoice::Always),
            _ => Err(anyhow::anyhow!("Invalid color choice: {}", s)),
        }
    }
}

fn main() -> Result<()> {
    let config = parse_args()?;
    
    if !config.quiet {
        eprintln!("cargo-builder: Running build with errors-only output...");
    }
    
    let exit_code = runner::run_build(&config)?;
    std::process::exit(exit_code);
}

fn separate_arguments(args: &[String]) -> (Vec<String>, Vec<String>) {
    // Define our tool's flags that take values
    let tool_flags_with_values = [
        "--log",
        "--log-color", 
        "--terminal-color",
    ];

    // Define our tool's boolean flags
    let tool_boolean_flags = [
        "--log-on-success",
        "--include-warnings", 
        "--show-build-output",
        "--quiet", "-q",
        "--help", "-h",
        "--version", "-V",
    ];

    let mut tool_args = Vec::new();
    let mut cargo_args = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];
        
        // Handle special case: explicit separator
        if arg == "--" {
            // Everything after -- goes to cargo
            cargo_args.extend_from_slice(&args[i + 1..]);
            break;
        }

        // Check if this is one of our tool flags
        if tool_boolean_flags.contains(&arg.as_str()) {
            tool_args.push(arg.clone());
            i += 1;
        } else if tool_flags_with_values.contains(&arg.as_str()) {
            tool_args.push(arg.clone());
            // Take the next argument as the value
            if i + 1 < args.len() {
                tool_args.push(args[i + 1].clone());
                i += 2;
            } else {
                // Flag without value, let clap handle the error
                tool_args.push(arg.clone());
                i += 1;
            }
        } else {
            // Not one of our flags, must be a cargo argument
            cargo_args.push(arg.clone());
            i += 1;
        }
    }

    (tool_args, cargo_args)
}

fn parse_args() -> Result<Config> {
    // Handle cargo subcommand - when called as "cargo builder", the first arg is "builder"
    let args: Vec<String> = env::args().collect();
    let raw_args = if args.len() > 1 && args[1] == "builder" {
        // Skip the "builder" subcommand argument
        args[2..].to_vec()
    } else {
        args[1..].to_vec()
    };

    // Separate our tool flags from cargo flags
    let (tool_args, cargo_args) = separate_arguments(&raw_args);

    // Parse our tool's arguments
    let matches = Command::new("cargo-builder")
        .about("A Cargo build wrapper that shows errors-only output with optional logging")
        .long_about("A Cargo build wrapper that shows errors-only output with optional logging.\n\nUsage:\n  cargo builder [OPTIONS] [cargo-build-args...]\n  cargo-builder [OPTIONS] [cargo-build-args...]")
        .version("0.1.0")
        .arg(
            Arg::new("log")
                .long("log")
                .value_name("PATH")
                .help("Target log file path")
        )
        .arg(
            Arg::new("log-on-success")
                .long("log-on-success")
                .action(ArgAction::SetTrue)
                .help("Keep the log file even on success")
        )
        .arg(
            Arg::new("log-color")
                .long("log-color")
                .value_name("CHOICE")
                .value_parser(["auto", "never", "always"])
                .default_value("never")
                .help("Color control for log file")
        )
        .arg(
            Arg::new("terminal-color")
                .long("terminal-color")
                .value_name("CHOICE")
                .value_parser(["auto", "never", "always"])
                .help("Color control for terminal output")
        )
        .arg(
            Arg::new("include-warnings")
                .long("include-warnings")
                .action(ArgAction::SetTrue)
                .help("Do not suppress rustc warnings")
        )
        .arg(
            Arg::new("show-build-output")
                .long("show-build-output")
                .action(ArgAction::SetTrue)
                .help("Show all raw cargo output (for debugging cargo issues)")
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Minimize plugin output")
        )
        .try_get_matches_from(std::iter::once("cargo-builder".to_string()).chain(tool_args))?;

    let config = Config {
        log_path: matches.get_one::<String>("log").cloned(),
        log_on_success: matches.get_flag("log-on-success"),
        log_color: matches.get_one::<String>("log-color")
            .unwrap()
            .parse()?,
        terminal_color: matches.get_one::<String>("terminal-color")
            .map(|s| s.parse())
            .transpose()?
            .unwrap_or(ColorChoice::Auto),
        include_warnings: matches.get_flag("include-warnings"),
        show_build_output: matches.get_flag("show-build-output"),
        quiet: matches.get_flag("quiet"),
        cargo_args,
    };

    Ok(config)
}