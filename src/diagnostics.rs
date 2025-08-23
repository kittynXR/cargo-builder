use crate::{Config, ColorChoice};
use anyhow::{Result, Context};
use serde_json::Value;
use regex::Regex;

#[derive(Debug)]
pub enum CargoMessage {
    CompilerMessage {
        level: String,
        rendered: String,
    },
    BuildFinished {
        success: bool,
    },
}

pub fn parse_cargo_message(line: &str) -> Result<Option<CargoMessage>> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }

    let json: Value = match serde_json::from_str(line) {
        Ok(json) => json,
        Err(_) => return Ok(None), // Not JSON, ignore
    };

    let reason = json.get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("");

    match reason {
        "compiler-message" => {
            let message = json.get("message")
                .context("Missing 'message' field in compiler-message")?;

            let level = message.get("level")
                .and_then(|l| l.as_str())
                .unwrap_or("unknown")
                .to_string();

            let rendered = message.get("rendered")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .to_string();

            if !rendered.is_empty() {
                Ok(Some(CargoMessage::CompilerMessage { level, rendered }))
            } else {
                Ok(None)
            }
        }
        "build-finished" => {
            let success = json.get("success")
                .and_then(|s| s.as_bool())
                .unwrap_or(false);

            Ok(Some(CargoMessage::BuildFinished { success }))
        }
        _ => Ok(None), // Not a message type we care about
    }
}

pub fn format_for_terminal(rendered: &str, config: &Config) -> String {
    match config.terminal_color {
        ColorChoice::Never => strip_ansi_codes(rendered),
        ColorChoice::Always => rendered.to_string(),
        ColorChoice::Auto => {
            if crate::term::should_use_color() {
                rendered.to_string()
            } else {
                strip_ansi_codes(rendered)
            }
        }
    }
}

pub fn format_for_log(rendered: &str, config: &Config) -> String {
    match config.log_color {
        ColorChoice::Always => rendered.to_string(),
        ColorChoice::Never => strip_ansi_codes(rendered),
        ColorChoice::Auto => {
            // For logs, default to no color unless explicitly requested
            strip_ansi_codes(rendered)
        }
    }
}

fn strip_ansi_codes(text: &str) -> String {
    lazy_static::lazy_static! {
        static ref ANSI_REGEX: Regex = Regex::new(r"\x1b\[[0-9;]*[mGKH]").unwrap();
    }
    ANSI_REGEX.replace_all(text, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compiler_message() {
        let json_line = r#"{"reason":"compiler-message","package_id":"test 0.1.0 (path+file:///tmp/test)","manifest_path":"/tmp/test/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"test","src_path":"/tmp/test/src/main.rs","edition":"2021","doc":false,"doctest":false,"test":false},"message":{"message":"cannot find value `undefined_var` in this scope","code":{"code":"E0425","explanation":""},"level":"error","spans":[{"file_name":"src/main.rs","byte_start":17,"byte_end":30,"line_start":2,"line_end":2,"column_start":20,"column_end":33,"is_primary":true,"text":[{"text":"    println!(\"{}\", undefined_var);","highlight_start":20,"highlight_end":33}],"label":"not found in this scope","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0425]: cannot find value `undefined_var` in this scope\n --> src/main.rs:2:20\n  |\n2 |     println!(\"{}\", undefined_var);\n  |                    ^^^^^^^^^^^^^ not found in this scope\n\n"}}"#;

        let result = parse_cargo_message(json_line).unwrap();
        match result {
            Some(CargoMessage::CompilerMessage { level, rendered }) => {
                assert_eq!(level, "error");
                assert!(rendered.contains("cannot find value `undefined_var`"));
            }
            _ => panic!("Expected CompilerMessage"),
        }
    }

    #[test]
    fn test_parse_build_finished() {
        let json_line = r#"{"reason":"build-finished","success":false}"#;
        
        let result = parse_cargo_message(json_line).unwrap();
        match result {
            Some(CargoMessage::BuildFinished { success }) => {
                assert_eq!(success, false);
            }
            _ => panic!("Expected BuildFinished"),
        }
    }

    #[test]
    fn test_strip_ansi_codes() {
        let text_with_ansi = "\x1b[31merror\x1b[0m: something went wrong";
        let stripped = strip_ansi_codes(text_with_ansi);
        assert_eq!(stripped, "error: something went wrong");
    }

    #[test]
    fn test_non_json_line() {
        let non_json = "This is not JSON";
        let result = parse_cargo_message(non_json).unwrap();
        assert!(result.is_none());
    }
}