use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Result};
use clap::{ArgAction, Parser};
use thiserror::Error;

#[derive(Parser)]
#[command(name = "plm-put-playlist")]
#[command(about = "Copy playlist files and associated media files from PC to device")]
#[command(version)]
struct Cli {
    /// Print verbose messages
    #[arg(short = 'v', long = "verbose", action = ArgAction::SetTrue)]
    verbose: bool,

    /// Destination to put playlists and media files into
    #[arg(required = true)]
    dest: String,

    /// Playlist file(s) to put
    #[arg(required = true)]
    playlists: Vec<String>,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to get absolute path: {0}")]
    AbsPath(String),
}

/// Get the absolute path of a directory
fn abs_dir(path: &str) -> Result<String, AppError> {
    let path = Path::new(path);
    let abs_path = fs::canonicalize(path).map_err(|e| {
        AppError::AbsPath(format!(
            "Failed to get absolute path for {}: {}",
            path.display(),
            e
        ))
    })?;

    if !abs_path.is_dir() {
        return Err(AppError::AbsPath(format!(
            "{} is not a directory",
            abs_path.display()
        )));
    }

    Ok(abs_path.to_string_lossy().to_string())
}

/// Print a message if verbose mode is enabled
fn print_message(verbose: bool, fmt: &str, args: &[&str]) {
    if verbose {
        let message = args
            .iter()
            .fold(fmt.to_string(), |acc, arg| acc.replacen("{}", arg, 1));
        eprintln!("{}", message);
    }
}

/// Copy media files from source to destination
fn copy_media_files(
    src_basedir: &str,
    dest_basedir: &str,
    files: impl Iterator<Item = String>,
    verbose: bool,
) -> Result<usize> {
    let mut n_files = 0;

    for file in files {
        let file_path = Path::new(&file);
        let dir_part = file_path.parent().unwrap_or(Path::new(""));
        let file_part = file_path.file_name().unwrap_or_default();

        let dest_dir = Path::new(dest_basedir).join(dir_part);

        if !dest_dir.exists() {
            fs::create_dir_all(&dest_dir)
                .with_context(|| format!("Failed to create directory: {}", dest_dir.display()))?;
        }

        let src_file = Path::new(src_basedir).join(&file);
        let dest_file = dest_dir.join(file_part);

        print_message(
            verbose,
            "Copy \"{}\" to \"{}\"",
            &[&src_file.to_string_lossy(), &dest_file.to_string_lossy()],
        );

        fs::copy(&src_file, &dest_file).with_context(|| {
            format!(
                "Failed to copy {} to {}",
                src_file.display(),
                dest_file.display()
            )
        })?;

        n_files += 1;
    }

    Ok(n_files)
}

/// Copy a playlist file and its associated media files
fn copy_playlist(playlist: &str, dest_basedir: &str, verbose: bool) -> Result<usize> {
    let playlist_path = Path::new(playlist);
    let src_basedir = playlist_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    let dest_dir = PathBuf::from(dest_basedir);
    if !dest_dir.exists() {
        fs::create_dir_all(&dest_dir)
            .with_context(|| format!("Failed to create directory: {}", dest_dir.display()))?;
    }

    let playlist_filename = playlist_path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid playlist filename"))?;

    let dest_playlist = dest_dir.join(playlist_filename);

    // Check if the playlist contains backslashes
    let playlist_content = fs::read_to_string(playlist)
        .with_context(|| format!("Failed to read playlist: {}", playlist))?;

    let has_backslashes = playlist_content
        .lines()
        .any(|line| !line.starts_with('#') && line.contains('\\'));

    if has_backslashes {
        // Replace backslashes with forward slashes
        let modified_content = playlist_content
            .lines()
            .map(|line| {
                if !line.starts_with('#') && line.contains('\\') {
                    line.replace('\\', "/")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&dest_playlist, modified_content)
            .with_context(|| format!("Failed to write playlist: {}", dest_playlist.display()))?;
    } else {
        print_message(
            verbose,
            "Copy playlist \"{}\" into \"{}\"",
            &[playlist, &format!("{}/", dest_basedir)],
        );

        fs::copy(playlist, &dest_playlist).with_context(|| {
            format!("Failed to copy {} to {}", playlist, dest_playlist.display())
        })?;
    }

    // Process media files
    let file =
        File::open(playlist).with_context(|| format!("Failed to open playlist: {}", playlist))?;
    let reader = BufReader::new(file);

    let media_files = reader
        .lines()
        .filter_map(Result::ok)
        .map(|line| {
            // Remove BOM if present
            let line = if line.starts_with('\u{feff}') {
                line[3..].to_string()
            } else {
                line
            };

            // Remove carriage return if present
            let line = if line.ends_with('\r') {
                line[..line.len() - 1].to_string()
            } else {
                line
            };

            line
        })
        .filter(|line| {
            // Skip comments and empty lines
            if line.starts_with('#') || line.is_empty() {
                return false;
            }
            true
        })
        .map(|line| {
            // Replace backslashes with forward slashes
            line.replace('\\', "/")
        });

    let n_files = copy_media_files(&src_basedir, dest_basedir, media_files, verbose)?;

    Ok(n_files)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let dest_dir = match abs_dir(&cli.dest) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(255);
        }
    };

    let mut n_playlists = 0;
    let mut n_files = 0;

    for playlist in &cli.playlists {
        print_message(
            cli.verbose,
            "Put playlist \"{}\" into \"{}\"",
            &[playlist, &dest_dir],
        );

        match copy_playlist(playlist, &dest_dir, cli.verbose) {
            Ok(files) => {
                n_files += files;
                n_playlists += 1;
            }
            Err(e) => {
                eprintln!("Error processing playlist {}: {}", playlist, e);
                process::exit(1);
            }
        }
    }

    println!("Number of copied playlists: {}", n_playlists);
    println!("Number of copied media files: {}", n_files);

    Ok(())
}
