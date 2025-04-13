use std::collections::HashSet;
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

    /// Copy lyrics files (.lrc) along with media files
    #[arg(short = 'l', long = "lyrics", action = ArgAction::SetTrue)]
    lyrics: bool,

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
    copy_lyrics: bool,
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

        // If lyrics option is enabled, try to copy the corresponding .lrc file
        if copy_lyrics {
            if let Some(stem) = file_path.file_stem() {
                let lyrics_filename = format!("{}.lrc", stem.to_string_lossy());
                let lyrics_path = Path::new(src_basedir).join(dir_part).join(&lyrics_filename);

                if lyrics_path.exists() {
                    let dest_lyrics_file = dest_dir.join(&lyrics_filename);

                    print_message(
                        verbose,
                        "Copy lyrics \"{}\" to \"{}\"",
                        &[&lyrics_path.to_string_lossy(), &dest_lyrics_file.to_string_lossy()],
                    );

                    fs::copy(&lyrics_path, &dest_lyrics_file).with_context(|| {
                        format!(
                            "Failed to copy lyrics {} to {}",
                            lyrics_path.display(),
                            dest_lyrics_file.display()
                        )
                    })?;

                    n_files += 1;
                }
            }
        }
    }

    Ok(n_files)
}

/// Extract media files from a playlist
fn extract_media_files(playlist: &str) -> Result<(String, Vec<String>)> {
    let playlist_path = Path::new(playlist);
    let src_basedir = playlist_path
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

    Ok((src_basedir, media_files))
}

/// Copy a playlist file to the destination
fn copy_playlist_file(playlist: &str, dest_basedir: &str, verbose: bool) -> Result<()> {
    let playlist_path = Path::new(playlist);
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

    Ok(())
}

/// Process a playlist file and its associated media files
fn process_playlist(
    playlist: &str,
    dest_basedir: &str,
    verbose: bool,
    media_files_map: &mut Vec<(String, HashSet<String>)>
) -> Result<(String, Vec<String>)> {
    print_message(
        verbose,
        "Processing playlist \"{}\"",
        &[playlist],
    );

    // Copy the playlist file
    copy_playlist_file(playlist, dest_basedir, verbose)?;

    // Extract media files
    let (src_basedir, files) = extract_media_files(playlist)?;

    // Add to the media files map
    let entry = media_files_map.iter_mut().find(|(base, _)| *base == src_basedir);

    if let Some((_, files_set)) = entry {
        // Add files to existing set
        for file in &files {
            files_set.insert(file.clone());
        }
    } else {
        // Create new entry
        let mut files_set = HashSet::new();
        for file in &files {
            files_set.insert(file.clone());
        }
        media_files_map.push((src_basedir.clone(), files_set));
    }

    Ok((src_basedir, files))
}

/// Filter out files that have already been copied
fn filter_already_copied_files(
    src_basedir: &str,
    files: &[String],
    copied_files: &HashSet<(String, String)>
) -> Vec<String> {
    files.iter()
        .filter(|file| !copied_files.contains(&(src_basedir.to_string(), file.to_string())))
        .cloned()
        .collect()
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
    let mut media_files_map: Vec<(String, HashSet<String>)> = Vec::new();
    let mut copied_files: HashSet<(String, String)> = HashSet::new();

    // Process each playlist and copy its media files one-by-one
    for playlist in &cli.playlists {
        print_message(
            cli.verbose,
            "Put playlist \"{}\" into \"{}\"",
            &[playlist, &dest_dir],
        );

        match process_playlist(playlist, &dest_dir, cli.verbose, &mut media_files_map) {
            Ok((src_basedir, files)) => {
                n_playlists += 1;

                // Filter out already copied files
                let files_to_copy = filter_already_copied_files(&src_basedir, &files, &copied_files);

                print_message(
                    cli.verbose,
                    "Copying {} media files for playlist \"{}\"",
                    &[&files_to_copy.len().to_string(), playlist],
                );

                // Copy files for this playlist
                match copy_media_files(&src_basedir, &dest_dir, files_to_copy.into_iter(), cli.verbose, cli.lyrics) {
                    Ok(copied) => {
                        n_files += copied;

                        // Update copied_files set
                        for file in files {
                            copied_files.insert((src_basedir.clone(), file));
                        }
                    }
                    Err(e) => {
                        eprintln!("Error copying media files for playlist {}: {}", playlist, e);
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

    println!("Number of copied playlists: {}", n_playlists);
    println!("Number of copied media files: {}", n_files);

    Ok(())
}
