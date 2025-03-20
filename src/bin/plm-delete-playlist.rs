use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process;

use anyhow::{Context, Result};
use clap::{ArgAction, Parser};
use thiserror::Error;

#[derive(Parser)]
#[command(name = "plm-delete-playlist")]
#[command(about = "Delete playlist files and associated media files from device")]
#[command(version)]
struct Cli {
    /// Print verbose messages
    #[arg(short = 'v', long = "verbose", action = ArgAction::SetTrue)]
    verbose: bool,

    /// Delete media files (and lyrics files with .lrc extension) associated with the playlist
    #[arg(short = 'm', long = "media", action = ArgAction::SetTrue)]
    media: bool,

    /// Playlist file(s) to delete
    #[arg(required = true)]
    playlists: Vec<String>,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
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

/// Extract media files from a playlist
fn extract_media_files(playlist: &str) -> Result<(String, Vec<String>)> {
    let playlist_path = Path::new(playlist);
    let base_dir = playlist_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

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
        })
        .collect();

    Ok((base_dir, media_files))
}

/// Delete a playlist file
fn delete_playlist_file(playlist: &str, verbose: bool) -> Result<()> {
    print_message(
        verbose,
        "Deleting playlist \"{}\"",
        &[playlist],
    );

    fs::remove_file(playlist)
        .with_context(|| format!("Failed to delete playlist: {}", playlist))?;

    Ok(())
}

/// Delete media files referenced in a playlist
fn delete_media_files(
    base_dir: &str,
    files: impl Iterator<Item = String>,
    verbose: bool,
) -> Result<usize> {
    let mut n_files = 0;

    for file in files {
        let file_path = Path::new(&file);
        let dir_part = file_path.parent().unwrap_or(Path::new(""));
        let file_stem = file_path.file_stem().unwrap_or_default();

        let media_file = Path::new(base_dir).join(&file);

        if media_file.exists() {
            print_message(
                verbose,
                "Deleting media file \"{}\"",
                &[&media_file.to_string_lossy()],
            );

            fs::remove_file(&media_file)
                .with_context(|| format!("Failed to delete media file: {}", media_file.display()))?;

            n_files += 1;
        } else if verbose {
            eprintln!("Media file not found: {}", media_file.display());
        }

        // Check for lyrics file with .lrc extension
        let lyrics_filename = format!("{}.lrc", file_stem.to_string_lossy());
        let lyrics_path = Path::new(base_dir).join(dir_part).join(&lyrics_filename);

        if lyrics_path.exists() {
            print_message(
                verbose,
                "Deleting lyrics file \"{}\"",
                &[&lyrics_path.to_string_lossy()],
            );

            fs::remove_file(&lyrics_path)
                .with_context(|| format!("Failed to delete lyrics file: {}", lyrics_path.display()))?;

            n_files += 1;
        }
    }

    Ok(n_files)
}

/// Delete empty directories recursively
fn delete_empty_dirs(dir: &Path, verbose: bool) -> Result<()> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }

    // First, recursively delete empty subdirectories
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            delete_empty_dirs(&path, verbose)?;
        }
    }

    // Check if directory is now empty
    let is_empty = fs::read_dir(dir)?.next().is_none();

    if is_empty {
        print_message(
            verbose,
            "Deleting empty directory \"{}\"",
            &[&dir.to_string_lossy()],
        );

        fs::remove_dir(dir)
            .with_context(|| format!("Failed to delete directory: {}", dir.display()))?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut media_files_map: Vec<(String, HashSet<String>)> = Vec::new();
    let mut n_playlists = 0;

    // First, process all playlists and collect media files
    for playlist in &cli.playlists {
        print_message(
            cli.verbose,
            "Processing playlist \"{}\"",
            &[playlist],
        );

        // Extract media files before deleting the playlist
        match extract_media_files(playlist) {
            Ok((base_dir, files)) => {
                // Add to the media files map
                let entry = media_files_map.iter_mut().find(|(base, _)| *base == base_dir);
                if let Some((_, files_set)) = entry {
                    // Add files to existing set
                    for file in files {
                        files_set.insert(file);
                    }
                } else {
                    // Create new entry
                    let mut files_set = HashSet::new();
                    for file in files {
                        files_set.insert(file);
                    }
                    media_files_map.push((base_dir, files_set));
                }

                // Delete the playlist file
                match delete_playlist_file(playlist, cli.verbose) {
                    Ok(_) => {
                        n_playlists += 1;
                    }
                    Err(e) => {
                        eprintln!("Error deleting playlist {}: {}", playlist, e);
                        process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error processing playlist {}: {}", playlist, e);
                process::exit(1);
            }
        }
    }

    // Now delete all unique media files if requested
    let mut n_files = n_playlists; // Start with number of playlists deleted

    if cli.media {
        print_message(
            cli.verbose,
            "Deleting {} unique media files",
            &[&media_files_map.iter().map(|(_, files)| files.len()).sum::<usize>().to_string()],
        );

        for (base_dir, files) in media_files_map {
            match delete_media_files(&base_dir, files.into_iter(), cli.verbose) {
                Ok(files_deleted) => {
                    n_files += files_deleted;
                }
                Err(e) => {
                    eprintln!("Error deleting media files: {}", e);
                    process::exit(1);
                }
            }

            // Delete empty directories
            let base_dir_path = Path::new(&base_dir);
            if let Err(e) = delete_empty_dirs(base_dir_path, cli.verbose) {
                eprintln!("Error deleting empty directories: {}", e);
                // Continue execution even if directory deletion fails
            }
        }
    }

    if cli.verbose {
        println!("Number of deleted files: {}", n_files);
    }

    Ok(())
}
