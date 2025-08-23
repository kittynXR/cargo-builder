use anyhow::{Result, Context};
use cargo_metadata::MetadataCommand;
use std::path::PathBuf;
use std::env;

pub struct Workspace {
    pub root: PathBuf,
    pub target_directory: PathBuf,
}

pub fn find_workspace() -> Result<Workspace> {
    let mut metadata_cmd = MetadataCommand::new();
    
    // Start from current directory
    let current_dir = env::current_dir()
        .context("Failed to get current directory")?;
    metadata_cmd.current_dir(&current_dir);
    
    // Don't fetch dependencies to make this faster
    metadata_cmd.no_deps();
    
    let metadata = metadata_cmd.exec()
        .context("Failed to get cargo metadata. Are you in a Rust project?")?;

    Ok(Workspace {
        root: metadata.workspace_root.into(),
        target_directory: metadata.target_directory.into(),
    })
}

pub fn is_in_workspace() -> bool {
    MetadataCommand::new()
        .no_deps()
        .exec()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_workspace_in_rust_project() {
        // This test will work if run from within the cargo-builder project
        if is_in_workspace() {
            let workspace = find_workspace().unwrap();
            assert!(workspace.root.exists());
            assert!(workspace.target_directory.is_absolute());
        }
    }

    #[test]
    fn test_is_in_workspace() {
        // Create a temporary directory with a Cargo.toml to test workspace detection
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        
        fs::write(&cargo_toml, r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#).unwrap();

        // Change to the temp directory
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();
        
        // Should detect workspace now
        assert!(is_in_workspace());
        
        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
    }

    #[test] 
    fn test_not_in_workspace() {
        // Create a temporary directory without Cargo.toml
        let temp_dir = TempDir::new().unwrap();
        
        // Change to the temp directory
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();
        
        // Should not detect workspace
        assert!(!is_in_workspace());
        
        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
    }
}