use crate::{Config, diagnostics};
use anyhow::{Result, Context};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct Logger {
    log_path: PathBuf,
    file: Option<File>,
    config: Config,
    has_written: bool,
}

impl Logger {
    pub fn new(log_path: &str, config: &Config) -> Result<Self> {
        Ok(Logger {
            log_path: PathBuf::from(log_path),
            file: None,
            config: config.clone(),
            has_written: false,
        })
    }

    pub fn log_error(&mut self, rendered: &str) -> Result<()> {
        // Initialize file on first error
        if self.file.is_none() {
            self.ensure_parent_dir()?;
            
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true) // Overwrite existing file
                .open(&self.log_path)
                .with_context(|| format!("Failed to create log file: {}", self.log_path.display()))?;
            
            self.file = Some(file);
            
            // Write header
            if let Some(ref mut f) = self.file {
                writeln!(f, "cargo-builder error log")?;
                writeln!(f, "======================")?;
                writeln!(f)?;
            }
        }

        // Format the message for the log file
        let log_content = diagnostics::format_for_log(rendered, &self.config);

        if let Some(ref mut file) = self.file {
            writeln!(file, "{}", log_content)?;
            writeln!(file)?; // Add blank line between errors
            file.flush()?;
            self.has_written = true;
        }

        Ok(())
    }

    pub fn finalize(self, build_success: bool) -> Result<()> {
        // Drop the file handle first
        drop(self.file);

        // Delete the log file if build succeeded and we're not keeping it
        if build_success && !self.config.log_on_success && self.has_written {
            if self.log_path.exists() {
                std::fs::remove_file(&self.log_path)
                    .with_context(|| format!("Failed to remove log file: {}", self.log_path.display()))?;
            }
        }

        Ok(())
    }

    fn ensure_parent_dir(&self) -> Result<()> {
        if let Some(parent) = self.log_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create log directory: {}", parent.display()))?;
            }
        }
        Ok(())
    }
}

// Need to implement Clone for Config to use it in Logger
impl Clone for crate::Config {
    fn clone(&self) -> Self {
        Self {
            log_path: self.log_path.clone(),
            log_on_success: self.log_on_success,
            log_color: self.log_color.clone(),
            terminal_color: self.terminal_color.clone(),
            include_warnings: self.include_warnings,
            show_build_output: self.show_build_output,
            quiet: self.quiet,
            cargo_args: self.cargo_args.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config() -> Config {
        Config {
            log_path: None,
            log_on_success: false,
            log_color: crate::ColorChoice::Never,
            terminal_color: crate::ColorChoice::Auto,
            include_warnings: false,
            show_build_output: false,
            quiet: false,
            cargo_args: vec![],
        }
    }

    #[test]
    fn test_logger_creates_file_on_first_error() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = create_test_config();
        
        let mut logger = Logger::new(log_path.to_str().unwrap(), &config).unwrap();
        
        assert!(!log_path.exists());
        
        logger.log_error("Test error message").unwrap();
        
        assert!(log_path.exists());
        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Test error message"));
        assert!(content.contains("cargo-builder error log"));
    }

    #[test]
    fn test_logger_removes_file_on_success() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = create_test_config();
        
        let mut logger = Logger::new(log_path.to_str().unwrap(), &config).unwrap();
        logger.log_error("Test error").unwrap();
        
        assert!(log_path.exists());
        
        // Finalize with success - should remove file
        logger.finalize(true).unwrap();
        
        assert!(!log_path.exists());
    }

    #[test]
    fn test_logger_keeps_file_on_failure() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = create_test_config();
        
        let mut logger = Logger::new(log_path.to_str().unwrap(), &config).unwrap();
        logger.log_error("Test error").unwrap();
        
        assert!(log_path.exists());
        
        // Finalize with failure - should keep file
        logger.finalize(false).unwrap();
        
        assert!(log_path.exists());
    }

    #[test]
    fn test_logger_keeps_file_with_log_on_success() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let mut config = create_test_config();
        config.log_on_success = true;
        
        let mut logger = Logger::new(log_path.to_str().unwrap(), &config).unwrap();
        logger.log_error("Test error").unwrap();
        
        assert!(log_path.exists());
        
        // Finalize with success but log_on_success=true - should keep file
        logger.finalize(true).unwrap();
        
        assert!(log_path.exists());
    }
}