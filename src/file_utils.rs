//! File utilities for generic file operations

use std::fs;
use std::path::Path;
// Context trait is used via method calls (.context()), suppress unused warning
#[allow(unused_imports)]
use anyhow::{Context, Result};

/// Creates a directory if it doesn't exist.
pub fn create_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Copies a file from the source path to the destination path.
pub fn copy_file(src_path: &Path, dest_path: &Path) -> Result<()> {
    // Create destination directory if it doesn't exist
    if let Some(dest_dir) = dest_path.parent() {
        if !dest_dir.exists() {
            fs::create_dir_all(dest_dir)?;
        }
    }

    // Attempt to copy the file
    fs::copy(src_path, dest_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_directory_success() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path().join("test_dir");

        // Test directory creation
        create_directory(&dir_path)?;

        assert!(dir_path.exists());

        Ok(())
    }

    #[test]
    fn test_copy_file_success() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let src_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("dest.txt");

        // Create source file
        fs::write(&src_file, "test content")?;

        // Test successful copy
        copy_file(&src_file, &dest_file)?;

        assert!(dest_file.exists());
        assert_eq!(fs::read_to_string(&dest_file)?, "test content");

        Ok(())
    }

    #[test]
    fn test_copy_file_creates_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let src_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("subdir").join("dest.txt");

        // Create source file
        fs::write(&src_file, "test content")?;

        // Test copy with directory creation
        copy_file(&src_file, &dest_file)?;

        assert!(dest_file.exists());
        assert_eq!(fs::read_to_string(&dest_file)?, "test content");

        Ok(())
    }
}
