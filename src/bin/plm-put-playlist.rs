use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
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

    /// Continue operation despite errors
    #[arg(short = 'k', long = "keep-going", action = ArgAction::SetTrue)]
    keep_going: bool,

    /// Write list of failed files to specified file (only with --keep-going)
    #[arg(short = 'e', long = "error-files", value_name = "FILE")]
    error_files: Option<String>,

    /// Retry failed operations from error file
    #[arg(short = 'r', long = "retry", value_name = "FILE")]
    retry_file: Option<String>,

    /// Destination to put playlists and media files into
    #[arg(required = true)]
    dest: String,

    /// Playlist file(s) to put
    #[arg(required_unless_present = "retry_file")]
    playlists: Vec<String>,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to get absolute path: {0}")]
    AbsPath(String),
}

/// Enum to represent different types of failures
enum FailureType {
    Playlist(String),          // Failed playlist path
    MediaFile(String, String), // (src_basedir, file) for failed media file
}

/// Struct to track failed files
struct ErrorTracker {
    failures: Vec<FailureType>, // Failures in operation order
}

impl ErrorTracker {
    fn new() -> Self {
        Self {
            failures: Vec::new(),
        }
    }

    fn add_failed_playlist(&mut self, playlist: String) {
        self.failures.push(FailureType::Playlist(playlist));
    }

    fn add_failed_media_file(&mut self, src_basedir: String, file: String) {
        self.failures
            .push(FailureType::MediaFile(src_basedir, file));
    }

    fn write_to_file(&self, path: &str) -> Result<(), io::Error> {
        let mut file = File::create(path)?;

        // Write failures in operation order with appropriate prefixes
        for failure in &self.failures {
            match failure {
                FailureType::Playlist(playlist) => {
                    writeln!(file, "P {}", playlist)?;
                }
                FailureType::MediaFile(src_basedir, file_path) => {
                    let full_path = Path::new(src_basedir).join(file_path);
                    writeln!(file, "M {}", full_path.display())?;
                }
            }
        }

        Ok(())
    }
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
fn print_message(
    verbose: bool,
    fmt: &str,
    args: &[&str],
    current_count: Option<usize>,
    total_count: Option<usize>,
    file_type: Option<&str>,
    copy_lyrics: bool,
) {
    if verbose {
        let message = if let (Some(current), Some(total)) = (current_count, total_count) {
            // Format with counters
            let counter_prefix = if let Some(ftype) = file_type {
                if ftype == "lyrics" && copy_lyrics {
                    format!("({}-L/{})", current, total)
                } else if ftype == "media" && copy_lyrics {
                    format!("({}-M/{})", current, total)
                } else {
                    format!("({}/{})", current, total)
                }
            } else {
                format!("({}/{})", current, total)
            };

            let formatted_message = args
                .iter()
                .fold(fmt.to_string(), |acc, arg| acc.replacen("{}", arg, 1));
            format!("{} {}", counter_prefix, formatted_message)
        } else {
            // Original format without counters
            args.iter()
                .fold(fmt.to_string(), |acc, arg| acc.replacen("{}", arg, 1))
        };

        eprintln!("{}", message);
    }
}

/// Copy a single media file from source to destination
/// Returns a tuple of (number of files copied, whether the media file was successfully copied)
fn copy_single_media_file(
    src_basedir: &str,
    dest_basedir: &str,
    file: &str,
    _verbose: bool,
    copy_lyrics: bool,
    keep_going: bool,
    error_tracker: &mut Option<&mut ErrorTracker>,
    _current_file_num: Option<usize>,
    _total_files: Option<usize>,
) -> Result<(usize, bool)> {
    let mut n_files = 0;
    let file_path = Path::new(file);
    let dir_part = file_path.parent().unwrap_or(Path::new(""));
    let file_part = file_path.file_name().unwrap_or_default();

    let dest_dir = Path::new(dest_basedir).join(dir_part);

    if !dest_dir.exists() {
        match fs::create_dir_all(&dest_dir) {
            Ok(_) => {}
            Err(e) => {
                let err = anyhow::Error::new(e).context(format!(
                    "Failed to create directory: {}",
                    dest_dir.display()
                ));
                if keep_going {
                    eprintln!("Error: {}", err);
                    if let Some(tracker) = error_tracker {
                        tracker.add_failed_media_file(src_basedir.to_string(), file.to_string());
                    }
                    return Ok((0, false));
                } else {
                    return Err(err);
                }
            }
        }
    }

    let src_file = Path::new(src_basedir).join(file);
    let dest_file = dest_dir.join(file_part);

    // We'll print the message in copy_media_files after successful copy

    match fs::copy(&src_file, &dest_file) {
        Ok(_) => {
            n_files += 1;
        }
        Err(e) => {
            let err = anyhow::Error::new(e).context(format!(
                "Failed to copy {} to {}",
                src_file.display(),
                dest_file.display()
            ));
            if keep_going {
                eprintln!("Error: {}", err);
                if let Some(tracker) = error_tracker {
                    tracker.add_failed_media_file(src_basedir.to_string(), file.to_string());
                }
                return Ok((0, false));
            } else {
                return Err(err);
            }
        }
    }

    // If lyrics option is enabled, try to copy the corresponding .lrc file
    if copy_lyrics {
        if let Some(stem) = file_path.file_stem() {
            let lyrics_filename = format!("{}.lrc", stem.to_string_lossy());
            let lyrics_path = Path::new(src_basedir).join(dir_part).join(&lyrics_filename);

            if lyrics_path.exists() {
                let dest_lyrics_file = dest_dir.join(&lyrics_filename);

                // We'll print the message in copy_media_files after successful copy

                match fs::copy(&lyrics_path, &dest_lyrics_file) {
                    Ok(_) => {
                        n_files += 1;
                    }
                    Err(e) => {
                        let err = anyhow::Error::new(e).context(format!(
                            "Failed to copy lyrics {} to {}",
                            lyrics_path.display(),
                            dest_lyrics_file.display()
                        ));
                        if keep_going {
                            eprintln!("Error: {}", err);
                            // We don't track lyrics files in the error tracker
                        } else {
                            return Err(err);
                        }
                    }
                }
            }
        }
    }

    Ok((n_files, true))
}

/// Copy media files from source to destination
/// Returns a tuple of (number of files copied, list of successfully copied media files)
fn copy_media_files(
    src_basedir: &str,
    dest_basedir: &str,
    files: impl Iterator<Item = String>,
    verbose: bool,
    copy_lyrics: bool,
    keep_going: bool,
    error_tracker: &mut Option<&mut ErrorTracker>,
    total_files: Option<usize>,
    current_success_count: &mut usize,
) -> Result<(usize, Vec<String>)> {
    let mut n_files = 0;
    let mut successful_files = Vec::new();
    let files_vec: Vec<String> = files.collect();

    for file in files_vec.into_iter() {
        // We'll update current_file_num only if the copy is successful
        let _current_file_num: Option<usize> = None;
        match copy_single_media_file(
            src_basedir,
            dest_basedir,
            &file,
            verbose,
            copy_lyrics,
            keep_going,
            error_tracker,
            None, // We'll print the message after successful copy
            total_files,
        ) {
            Ok((copied, success)) => {
                n_files += copied;
                if success {
                    // Increment the global success counter
                    *current_success_count += 1;

                    // Print message with updated counter after successful copy
                    let src_file = Path::new(src_basedir).join(&file);
                    let file_path = Path::new(&file);
                    let dir_part = file_path.parent().unwrap_or(Path::new(""));
                    let file_part = file_path.file_name().unwrap_or_default();
                    let dest_file = Path::new(dest_basedir).join(dir_part).join(file_part);

                    print_message(
                        verbose,
                        "Copy track \"{}\" to \"{}\"",
                        &[&src_file.to_string_lossy(), &dest_file.to_string_lossy()],
                        Some(*current_success_count),
                        total_files,
                        Some("media"),
                        copy_lyrics,
                    );

                    // If lyrics option is enabled, print message for lyrics file too
                    if copy_lyrics {
                        if let Some(stem) = file_path.file_stem() {
                            let lyrics_filename = format!("{}.lrc", stem.to_string_lossy());
                            let lyrics_path =
                                Path::new(src_basedir).join(dir_part).join(&lyrics_filename);

                            if lyrics_path.exists() {
                                let dest_lyrics_file = Path::new(dest_basedir)
                                    .join(dir_part)
                                    .join(&lyrics_filename);

                                print_message(
                                    verbose,
                                    "Copy lyrics \"{}\" to \"{}\"",
                                    &[
                                        &lyrics_path.to_string_lossy(),
                                        &dest_lyrics_file.to_string_lossy(),
                                    ],
                                    Some(*current_success_count),
                                    total_files,
                                    Some("lyrics"),
                                    copy_lyrics,
                                );
                            }
                        }
                    }

                    successful_files.push(file);
                }
            }
            Err(e) => return Err(e),
        }
    }

    Ok((n_files, successful_files))
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
fn copy_playlist_file(
    playlist: &str,
    dest_basedir: &str,
    verbose: bool,
    current_playlist_num: Option<usize>,
    total_playlists: Option<usize>,
) -> Result<()> {
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
            "Copy playlist \"{}\" to \"{}\"",
            &[playlist, &format!("{}/", dest_basedir)],
            current_playlist_num,
            total_playlists,
            None,
            false,
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
    media_files_map: &mut Vec<(String, HashSet<String>)>,
    current_playlist_num: Option<usize>,
    total_playlists: Option<usize>,
) -> Result<(String, Vec<String>)> {
    print_message(
        verbose,
        "Processing playlist \"{}\"",
        &[playlist],
        None,
        None,
        None,
        false,
    );

    // Copy the playlist file
    copy_playlist_file(
        playlist,
        dest_basedir,
        verbose,
        current_playlist_num,
        total_playlists,
    )?;

    // Extract media files
    let (src_basedir, files) = extract_media_files(playlist)?;

    // Add to the media files map
    let entry = media_files_map
        .iter_mut()
        .find(|(base, _)| *base == src_basedir);

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
    copied_files: &HashSet<(String, String)>,
) -> Vec<String> {
    files
        .iter()
        .filter(|file| !copied_files.contains(&(src_basedir.to_string(), file.to_string())))
        .cloned()
        .collect()
}

/// Parse an error file and extract failed playlists and media files
fn parse_error_file(path: &str) -> Result<(Vec<String>, Vec<(String, String)>)> {
    let file = File::open(path).with_context(|| format!("Failed to open error file: {}", path))?;
    let reader = BufReader::new(file);

    let mut playlists = Vec::new();
    let mut media_files = Vec::new();

    println!("Parsing error file: {}", path);

    for line in reader.lines() {
        let line = line?;
        println!("  Line: {}", line);

        if line.starts_with("P ") {
            // Playlist entry
            let playlist = line[2..].trim().to_string();
            println!("    Found playlist: {}", playlist);
            playlists.push(playlist);
        } else if line.starts_with("M ") {
            // Media file entry
            let file_path = line[2..].trim().to_string();
            println!("    Found media file: {}", file_path);

            let path = Path::new(&file_path);

            // Extract the base directory (up to the MUSIC directory) and the relative path
            let path_str = path.to_string_lossy();
            if let Some(music_idx) = path_str.find("/MUSIC/") {
                // Extract the base directory (up to and including MUSIC)
                let src_basedir = &path_str[..music_idx + 7]; // +7 to include "/MUSIC/"

                // Extract the relative path (after MUSIC/)
                let rel_path = &path_str[music_idx + 7..];

                println!("      Base dir: {}", src_basedir);
                println!("      Relative path: {}", rel_path);

                if !rel_path.is_empty() {
                    media_files.push((src_basedir.to_string(), rel_path.to_string()));
                }
            } else {
                // Fallback to the old method if MUSIC directory is not found
                let src_basedir = path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| ".".to_string());

                let file_name = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();

                println!("      Base dir (fallback): {}", src_basedir);
                println!("      File name: {}", file_name);

                if !file_name.is_empty() {
                    media_files.push((src_basedir, file_name));
                }
            }
        }
        // Ignore any other lines
    }

    println!(
        "Parsed {} playlists and {} media files",
        playlists.len(),
        media_files.len()
    );

    Ok((playlists, media_files))
}

/// Retry processing a single playlist from the error file
fn retry_playlist(
    playlist: &str,
    dest_dir: &str,
    verbose: bool,
    lyrics: bool,
    keep_going: bool,
    error_tracker: &mut Option<&mut ErrorTracker>,
    media_files_map: &mut Vec<(String, HashSet<String>)>,
    copied_files: &mut HashSet<(String, String)>,
    current_playlist_num: Option<usize>,
    total_playlists: Option<usize>,
    total_media_files: Option<usize>,
    successful_media_files: &mut usize,
) -> Result<(bool, usize)> {
    print_message(
        verbose,
        "Retrying playlist \"{}\"",
        &[playlist],
        None,
        None,
        None,
        false,
    );

    match process_playlist(
        playlist,
        dest_dir,
        verbose,
        media_files_map,
        current_playlist_num,
        total_playlists,
    ) {
        Ok((src_basedir, files)) => {
            // Copy media files for this playlist
            let files_to_copy = filter_already_copied_files(&src_basedir, &files, copied_files);

            print_message(
                verbose,
                "Copying {} media files for playlist \"{}\"",
                &[&files_to_copy.len().to_string(), playlist],
                None,
                None,
                None,
                lyrics,
            );

            match copy_media_files(
                &src_basedir,
                dest_dir,
                files_to_copy.into_iter(),
                verbose,
                lyrics,
                keep_going,
                error_tracker,
                total_media_files,
                successful_media_files,
            ) {
                Ok((_, successful_files)) => {
                    let successful_count = successful_files.len();

                    // Update copied_files set
                    for file in successful_files {
                        copied_files.insert((src_basedir.clone(), file));
                    }

                    Ok((true, successful_count))
                }
                Err(e) => {
                    eprintln!("Error copying media files for playlist {}: {}", playlist, e);
                    if !keep_going {
                        return Err(e);
                    }
                    Ok((true, 0))
                }
            }
        }
        Err(e) => {
            eprintln!("Error processing playlist {}: {}", playlist, e);
            if let Some(tracker) = error_tracker {
                tracker.add_failed_playlist(playlist.to_string());
            }
            if !keep_going {
                return Err(e);
            }
            Ok((false, 0))
        }
    }
}

/// Retry copying a single media file from the error file
fn retry_media_file(
    src_basedir: &str,
    file: &str,
    dest_dir: &str,
    verbose: bool,
    lyrics: bool,
    keep_going: bool,
    error_tracker: &mut Option<&mut ErrorTracker>,
    copied_files: &mut HashSet<(String, String)>,
    _current_file_num: Option<usize>,
    total_media_files: Option<usize>,
    successful_media_files: &mut usize,
) -> Result<usize> {
    print_message(
        verbose,
        "Retrying media file \"{}\"",
        &[&Path::new(src_basedir).join(file).to_string_lossy()],
        None,
        None,
        None,
        false,
    );

    // Check if this file has already been copied
    if copied_files.contains(&(src_basedir.to_string(), file.to_string())) {
        print_message(
            verbose,
            "Skipping already copied file \"{}\"",
            &[&Path::new(src_basedir).join(file).to_string_lossy()],
            None,
            None,
            None,
            false,
        );
        return Ok(1);
    }

    // Copy the file
    match copy_media_files(
        src_basedir,
        dest_dir,
        std::iter::once(file.to_string()),
        verbose,
        lyrics,
        keep_going,
        error_tracker,
        total_media_files,
        successful_media_files,
    ) {
        Ok((_, successful_files)) => {
            let successful_count = successful_files.len();

            // Update copied_files set
            for file in successful_files {
                copied_files.insert((src_basedir.to_string(), file));
            }

            Ok(successful_count)
        }
        Err(e) => {
            eprintln!(
                "Error copying media file {}: {}",
                Path::new(src_basedir).join(file).display(),
                e
            );
            if !keep_going {
                return Err(e);
            }
            Ok(0)
        }
    }
}

/// Process retry operations from an error file
fn retry_operations(
    retry_file: &str,
    dest_dir: &str,
    verbose: bool,
    lyrics: bool,
    keep_going: bool,
    error_tracker: &mut Option<&mut ErrorTracker>,
) -> Result<(usize, usize, usize, usize)> {
    print_message(
        verbose,
        "Retrying operations from error file \"{}\"",
        &[retry_file],
        None,
        None,
        None,
        false,
    );

    let (playlists, media_files) = parse_error_file(retry_file)?;

    let total_playlists = playlists.len();
    let total_media_files = media_files.len();
    let mut successful_playlists = 0;
    let mut successful_media_files = 0;
    let mut media_files_map: Vec<(String, HashSet<String>)> = Vec::new();
    let mut copied_files: HashSet<(String, String)> = HashSet::new();

    // Process playlists first
    for (i, playlist) in playlists.iter().enumerate() {
        match retry_playlist(
            playlist,
            dest_dir,
            verbose,
            lyrics,
            keep_going,
            error_tracker,
            &mut media_files_map,
            &mut copied_files,
            Some(i + 1),
            Some(total_playlists),
            Some(total_media_files),
            &mut successful_media_files,
        ) {
            Ok((success, count)) => {
                if success {
                    successful_playlists += 1;
                }
                successful_media_files += count;
            }
            Err(e) => return Err(e),
        }
    }

    // Process media files
    for (i, (src_basedir, file)) in media_files.iter().enumerate() {
        match retry_media_file(
            src_basedir,
            file,
            dest_dir,
            verbose,
            lyrics,
            keep_going,
            error_tracker,
            &mut copied_files,
            Some(i + 1),
            Some(total_media_files),
            &mut successful_media_files,
        ) {
            Ok(count) => {
                successful_media_files += count;
            }
            Err(e) => return Err(e),
        }
    }

    Ok((
        successful_playlists,
        total_playlists,
        successful_media_files,
        total_media_files,
    ))
}

/// Process normal operations (non-retry mode)
fn process_normal_operations(
    playlists: &[String],
    dest_dir: &str,
    verbose: bool,
    lyrics: bool,
    keep_going: bool,
    error_tracker_ref: &mut Option<&mut ErrorTracker>,
) -> Result<(usize, usize, usize, usize)> {
    let total_playlists = playlists.len();
    let mut successful_playlists = 0;
    let mut successful_media_files = 0;
    let mut media_files_map: Vec<(String, HashSet<String>)> = Vec::new();
    let mut copied_files: HashSet<(String, String)> = HashSet::new();

    // First, calculate the total number of unique media files across all playlists
    let mut all_media_files: HashSet<(String, String)> = HashSet::new();

    // Process each playlist to extract media files and build the global map
    for playlist in playlists.iter() {
        match extract_media_files(playlist) {
            Ok((src_basedir, files)) => {
                for file in files {
                    all_media_files.insert((src_basedir.clone(), file));
                }
            }
            Err(e) => {
                eprintln!(
                    "Error extracting media files from playlist {}: {}",
                    playlist, e
                );
                if !keep_going {
                    return Err(e);
                }
            }
        }
    }

    // Total number of unique media files across all playlists
    let total_media_files = all_media_files.len();

    // Process each playlist and copy its media files one-by-one
    for (i, playlist) in playlists.iter().enumerate() {
        print_message(
            verbose,
            "Put playlist \"{}\" into \"{}\"",
            &[playlist, dest_dir],
            None,
            None,
            None,
            false,
        );

        match process_playlist(
            playlist,
            dest_dir,
            verbose,
            &mut media_files_map,
            Some(i + 1),
            Some(total_playlists),
        ) {
            Ok((src_basedir, files)) => {
                // Filter out already copied files
                let files_to_copy =
                    filter_already_copied_files(&src_basedir, &files, &copied_files);

                print_message(
                    verbose,
                    "Copying {} media files for playlist \"{}\"",
                    &[&files_to_copy.len().to_string(), playlist],
                    None,
                    None,
                    None,
                    lyrics,
                );

                // Copy files for this playlist
                match copy_media_files(
                    &src_basedir,
                    dest_dir,
                    files_to_copy.into_iter(),
                    verbose,
                    lyrics,
                    keep_going,
                    error_tracker_ref,
                    Some(total_media_files),
                    &mut successful_media_files,
                ) {
                    Ok((_copied, successful_files)) => {
                        // The successful_media_files counter is already updated in copy_media_files
                        // No need to increment it again here
                        successful_playlists += 1;

                        // Update copied_files set with only the successfully copied files
                        for file in successful_files {
                            copied_files.insert((src_basedir.clone(), file));
                        }
                    }
                    Err(e) => {
                        eprintln!("Error copying media files for playlist {}: {}", playlist, e);
                        if !keep_going {
                            process::exit(1);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error processing playlist {}: {}", playlist, e);
                if let Some(tracker) = error_tracker_ref {
                    tracker.add_failed_playlist(playlist.to_string());
                }
                if !keep_going {
                    process::exit(1);
                }
            }
        }
    }

    Ok((
        successful_playlists,
        total_playlists,
        successful_media_files,
        total_media_files,
    ))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Validate that --error-files is only used with --keep-going when not using --retry
    if cli.error_files.is_some() && !cli.keep_going && cli.retry_file.is_none() {
        eprintln!("Error: --error-files can only be used with --keep-going");
        process::exit(255);
    }

    // Validate that --retry and --error-files don't use the same file
    if let (Some(retry_file), Some(error_file)) = (&cli.retry_file, &cli.error_files) {
        if retry_file == error_file {
            eprintln!("Error: --retry and --error-files cannot specify the same file");
            process::exit(255);
        }
    }

    // Test if error file can be created (fail fast)
    if let Some(error_file) = &cli.error_files {
        match File::create(error_file) {
            Ok(_) => {
                // File can be created, we'll write to it at the end if needed
                // The file will remain empty if no errors occur
            }
            Err(e) => {
                eprintln!("Error: Failed to create error log file: {}", e);
                process::exit(2);
            }
        }
    }

    // Initialize error tracker if --error-files is specified
    let mut error_tracker: Option<ErrorTracker> =
        cli.error_files.as_ref().map(|_| ErrorTracker::new());
    let mut error_tracker_ref: Option<&mut ErrorTracker> = error_tracker.as_mut();

    let dest_dir = match abs_dir(&cli.dest) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(255);
        }
    };

    // Check if we're in retry mode
    if let Some(retry_file) = &cli.retry_file {
        // Process retry operations
        match retry_operations(
            retry_file,
            &dest_dir,
            cli.verbose,
            cli.lyrics,
            cli.keep_going,
            &mut error_tracker_ref,
        ) {
            Ok((
                successful_playlists,
                total_playlists,
                successful_media_files,
                total_media_files,
            )) => {
                println!(
                    "({}/{}) playlist copied",
                    successful_playlists, total_playlists
                );
                println!(
                    "({}/{}) media files copied",
                    successful_media_files, total_media_files
                );
            }
            Err(e) => {
                eprintln!("Error during retry operations: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Normal operation mode
        match process_normal_operations(
            &cli.playlists,
            &dest_dir,
            cli.verbose,
            cli.lyrics,
            cli.keep_going,
            &mut error_tracker_ref,
        ) {
            Ok((
                successful_playlists,
                total_playlists,
                successful_media_files,
                total_media_files,
            )) => {
                println!(
                    "({}/{}) playlist copied",
                    successful_playlists, total_playlists
                );
                println!(
                    "({}/{}) media files copied",
                    successful_media_files, total_media_files
                );
            }
            Err(e) => {
                eprintln!("Error during normal operations: {}", e);
                process::exit(1);
            }
        }
    }

    // Write error log if requested
    if let Some(error_file) = cli.error_files {
        if let Some(tracker) = error_tracker {
            if let Err(e) = tracker.write_to_file(&error_file) {
                eprintln!("Error: Failed to write error log file: {}", e);
                process::exit(2);
            }
        }
    }

    Ok(())
}
