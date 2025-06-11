//! File utilities for generic file operations

use std::fs;
use std::path::Path;
// Context trait is used via method calls (.context()), suppress unused warning
#[allow(unused_imports)]
use anyhow::{Context, Result};

/// Generic file copy function that handles directory creation and copying
/// Returns true if the copy was successful, false if it failed but should continue (via callback)
/// Returns Err if the operation should stop
pub fn generic_copy_file<F>(
    src_path: &Path,
    dest_path: &Path,
    on_error: F,
) -> Result<bool>
where
    F: FnOnce(&anyhow::Error) -> bool, // Returns true to continue, false to stop
{
    // Create destination directory if it doesn't exist
    if let Some(dest_dir) = dest_path.parent() {
        if !dest_dir.exists() {
            if let Err(e) = fs::create_dir_all(dest_dir) {
                let err = anyhow::Error::new(e).context(format!(
                    "Failed to create directory: {}",
                    dest_dir.display()
                ));
                return if on_error(&err) {
                    Ok(false) // Continue but mark as failed
                } else {
                    Err(err) // Stop execution
                };
            }
        }
    }

    // Attempt to copy the file
    match fs::copy(src_path, dest_path) {
        Ok(_) => Ok(true),
        Err(e) => {
            let err = anyhow::Error::new(e).context(format!(
                "Failed to copy {} to {}",
                src_path.display(),
                dest_path.display()
            ));
            if on_error(&err) {
                Ok(false) // Continue but mark as failed
            } else {
                Err(err) // Stop execution
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generic_copy_file_success() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let src_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("dest.txt");

        // Create source file
        fs::write(&src_file, "test content")?;

        // Test successful copy
        let result = generic_copy_file(
            &src_file,
            &dest_file,
            |_err| {
                panic!("Should not call error callback on success");
            },
        )?;

        assert!(result);
        assert!(dest_file.exists());
        assert_eq!(fs::read_to_string(&dest_file)?, "test content");

        Ok(())
    }

    #[test]
    fn test_generic_copy_file_creates_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let src_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("subdir").join("dest.txt");

        // Create source file
        fs::write(&src_file, "test content")?;

        // Test copy with directory creation
        let result = generic_copy_file(
            &src_file,
            &dest_file,
            |_err| {
                panic!("Should not call error callback on success");
            },
        )?;

        assert!(result);
        assert!(dest_file.exists());
        assert_eq!(fs::read_to_string(&dest_file)?, "test content");

        Ok(())
    }

    #[test]
    fn test_generic_copy_file_source_not_found_continue() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let src_file = temp_dir.path().join("nonexistent.txt");
        let dest_file = temp_dir.path().join("dest.txt");

        let mut error_called = false;

        // Test with continue on error
        let result = generic_copy_file(
            &src_file,
            &dest_file,
            |err| {
                error_called = true;
                assert!(err.to_string().contains("Failed to copy"));
                true // Continue
            },
        )?;

        assert!(!result);
        assert!(error_called);
        assert!(!dest_file.exists());

        Ok(())
    }

    #[test]
    fn test_generic_copy_file_source_not_found_stop() {
        let temp_dir = TempDir::new().unwrap();
        let src_file = temp_dir.path().join("nonexistent.txt");
        let dest_file = temp_dir.path().join("dest.txt");

        let mut error_called = false;

        // Test with stop on error
        let result = generic_copy_file(
            &src_file,
            &dest_file,
            |err| {
                error_called = true;
                assert!(err.to_string().contains("Failed to copy"));
                false // Stop
            },
        );

        assert!(result.is_err());
        assert!(error_called);
        assert!(!dest_file.exists());
    }

    #[test]
    fn test_generic_copy_file_directory_creation_fails_continue() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let src_file = temp_dir.path().join("source.txt");

        // Try to create dest file in a path that can't be created (using a file as directory)
        let blocking_file = temp_dir.path().join("blocking_file");
        fs::write(&blocking_file, "block")?;
        let dest_file = blocking_file.join("dest.txt"); // This should fail

        // Create source file
        fs::write(&src_file, "test content")?;

        let mut error_called = false;

        // Test with continue on directory creation error
        let result = generic_copy_file(
            &src_file,
            &dest_file,
            |err| {
                error_called = true;
                // The error could be either directory creation or file copy failure
                let err_str = err.to_string();
                assert!(err_str.contains("Failed to create directory") || err_str.contains("Failed to copy"));
                true // Continue
            },
        )?;

        assert!(!result);
        assert!(error_called);

        Ok(())
    }

    #[test]
    fn test_generic_copy_file_directory_creation_fails_stop() {
        let temp_dir = TempDir::new().unwrap();
        let src_file = temp_dir.path().join("source.txt");

        // Try to create dest file in a path that can't be created (using a file as directory)
        let blocking_file = temp_dir.path().join("blocking_file");
        fs::write(&blocking_file, "block").unwrap();
        let dest_file = blocking_file.join("dest.txt"); // This should fail

        // Create source file
        fs::write(&src_file, "test content").unwrap();

        let mut error_called = false;

        // Test with stop on directory creation error
        let result = generic_copy_file(
            &src_file,
            &dest_file,
            |err| {
                error_called = true;
                // The error could be either directory creation or file copy failure
                let err_str = err.to_string();
                assert!(err_str.contains("Failed to create directory") || err_str.contains("Failed to copy"));
                false // Stop
            },
        );

        assert!(result.is_err());
        assert!(error_called);
    }

    #[test]
    fn test_generic_copy_file_overwrites_existing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let src_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("dest.txt");

        // Create source and destination files
        fs::write(&src_file, "new content")?;
        fs::write(&dest_file, "old content")?;

        // Test overwriting existing file
        let result = generic_copy_file(
            &src_file,
            &dest_file,
            |_err| {
                panic!("Should not call error callback on success");
            },
        )?;

        assert!(result);
        assert_eq!(fs::read_to_string(&dest_file)?, "new content");

        Ok(())
    }

    #[test]
    fn test_generic_copy_file_dest_in_root() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let src_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("dest.txt");

        // Create source file
        fs::write(&src_file, "test content")?;

        // Test copy to root of temp directory (no subdirectory creation needed)
        let result = generic_copy_file(
            &src_file,
            &dest_file,
            |_err| {
                panic!("Should not call error callback on success");
            },
        )?;

        assert!(result);
        assert!(dest_file.exists());
        assert_eq!(fs::read_to_string(&dest_file)?, "test content");

        Ok(())
    }
}
