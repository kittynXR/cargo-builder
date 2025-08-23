use crate::{Config, ColorChoice, diagnostics, logging, util};
use anyhow::{Result, Context};
use std::process::{Command, Stdio};
use std::env;
use std::io::{BufRead, BufReader};

pub fn run_build(config: &Config) -> Result<i32> {
    let workspace = util::find_workspace()?;
    let log_path = config.log_path.clone()
        .unwrap_or_else(|| workspace.target_directory.join("build-errors.log").display().to_string());

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
       .arg("--message-format=json-diagnostic-rendered-ansi");

    // Add user's cargo args
    for arg in &config.cargo_args {
        cmd.arg(arg);
    }

    // Set up environment
    setup_environment(&mut cmd, config)?;

    // Configure stdio
    cmd.stdout(Stdio::piped());
    if config.show_build_output {
        cmd.stderr(Stdio::inherit());
    } else {
        cmd.stderr(Stdio::piped());
    }

    if !config.quiet {
        eprintln!("cargo-builder: Starting build...");
    }

    let mut child = cmd.spawn()
        .context("Failed to spawn cargo build process")?;

    let stdout = child.stdout.take()
        .context("Failed to capture stdout")?;
    let stderr = if !config.show_build_output {
        child.stderr.take()
    } else {
        None
    };

    let mut logger = logging::Logger::new(&log_path, config)?;
    let mut build_success = None;
    let mut has_errors = false;
    let mut captured_stderr = Vec::new();

    // Process stdout (JSON messages)
    let stdout_reader = BufReader::new(stdout);
    for line in stdout_reader.lines() {
        let line = line.context("Failed to read stdout line")?;
        
        match diagnostics::parse_cargo_message(&line)? {
            Some(diagnostics::CargoMessage::CompilerMessage { level, rendered }) => {
                match level.as_str() {
                    "error" => {
                        has_errors = true;
                        let terminal_output = diagnostics::format_for_terminal(&rendered, config);
                        eprint!("{}", terminal_output);
                        logger.log_error(&rendered)?;
                    }
                    "warning" if config.include_warnings => {
                        let terminal_output = diagnostics::format_for_terminal(&rendered, config);
                        eprint!("{}", terminal_output);
                        if config.log_on_success {
                            logger.log_error(&rendered)?;
                        }
                    }
                    _ => {} // Ignore other levels
                }
            }
            Some(diagnostics::CargoMessage::BuildFinished { success }) => {
                build_success = Some(success);
            }
            None => {} // Not a message we care about
        }
    }

    // Process stderr if we're capturing it
    if let Some(stderr) = stderr {
        let stderr_reader = BufReader::new(stderr);
        for line in stderr_reader.lines() {
            let line = line.context("Failed to read stderr line")?;
            captured_stderr.push(line);
        }
    }

    let exit_status = child.wait()
        .context("Failed to wait for cargo build process")?;

    let exit_code = exit_status.code().unwrap_or(1);
    let final_success = build_success.unwrap_or(exit_code == 0);

    // Handle case where build failed but we didn't capture error messages
    if !final_success && !has_errors && !captured_stderr.is_empty() {
        let stderr_content = captured_stderr.join("\n");
        eprintln!("Build failed; details in log file: {}", log_path);
        logger.log_error(&stderr_content)?;
        has_errors = true;
    }

    // Finalize logging
    logger.finalize(final_success && !has_errors)?;

    if !config.quiet {
        if final_success && !has_errors {
            eprintln!("cargo-builder: Build completed successfully");
        } else {
            eprintln!("cargo-builder: Build failed with errors");
            if has_errors {
                eprintln!("cargo-builder: Error details written to: {}", log_path);
            }
        }
    }

    Ok(exit_code)
}

fn setup_environment(cmd: &mut Command, config: &Config) -> Result<()> {
    // Handle RUSTFLAGS
    if !config.include_warnings {
        let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();
        if !rustflags.is_empty() {
            rustflags.push(' ');
        }
        rustflags.push_str("-Awarnings");
        cmd.env("RUSTFLAGS", rustflags);
    }

    // Handle CARGO_TERM_COLOR for terminal output
    match &config.terminal_color {
        ColorChoice::Always => cmd.env("CARGO_TERM_COLOR", "always"),
        ColorChoice::Never => cmd.env("CARGO_TERM_COLOR", "never"),
        ColorChoice::Auto => {
            // Let cargo decide based on TTY detection
            cmd
        }
    };

    Ok(())
}