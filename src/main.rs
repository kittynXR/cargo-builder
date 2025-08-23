mod runner;
mod diagnostics; 
mod logging;
mod term;
mod util;

use clap::{Arg, ArgAction, Command};
use anyhow::Result;

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

fn parse_args() -> Result<Config> {
    let matches = Command::new("cargo-builder")
        .about("A Cargo build wrapper that shows errors-only output with optional logging")
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
                .help("Also mirror Cargo's raw stderr")
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Minimize plugin output")
        )
        .arg(
            Arg::new("cargo-args")
                .last(true)
                .num_args(0..)
                .help("Arguments to pass to cargo build")
        )
        .get_matches();

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
        cargo_args: matches.get_many::<String>("cargo-args")
            .map(|vals| vals.cloned().collect())
            .unwrap_or_default(),
    };

    Ok(config)
}