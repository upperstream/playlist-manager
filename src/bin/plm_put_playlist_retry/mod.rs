use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context as AnyhowContext, Result};

// Import MediaFileInfo from the shared module
use playlist_manager::media_file_info::MediaFileInfo;

/// Struct to hold destination directory information
pub struct RetryContext {
    pub dest_dir: String,
}

/// Struct to hold media files map and copied files
pub struct MediaContext {
    pub media_files_map: Vec<(String, HashSet<String>)>,
    pub copied_files: HashSet<(String, String)>,
}

/// Struct to hold progress tracking information
pub struct ProgressContext {
    pub current_playlist_num: Option<usize>,
    pub total_playlists: Option<usize>,
    pub total_media_files: Option<usize>,
    pub successful_media_files: usize,
}

/// Parse an error file and extract failed playlists and media files
pub fn parse_error_file(path: &str) -> Result<(Vec<String>, Vec<(String, String)>)> {
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
pub fn retry_playlist(
    playlist: &str,
    retry_context: &RetryContext,
    options: &super::CommandOptions,
    error_tracker: &mut Option<&mut super::ErrorTracker>,
    media_context: &mut MediaContext,
    progress_context: &mut ProgressContext,
) -> Result<(bool, usize)> {
    super::print_message(
        options.verbose,
        "Retrying playlist \"{}\"",
        &[playlist],
        None,
        None,
        None,
    );

    match super::process_playlist(
        playlist,
        &retry_context.dest_dir,
        options.verbose,
        &mut media_context.media_files_map,
        progress_context.current_playlist_num,
        progress_context.total_playlists,
    ) {
        Ok((src_basedir, files)) => {
            // Copy media files for this playlist
            let files_to_copy = super::filter_already_copied_files(
                &src_basedir,
                &files,
                &media_context.copied_files,
            );

            super::print_message(
                options.verbose,
                "Copying {} media files for playlist \"{}\"",
                &[&files_to_copy.len().to_string(), playlist],
                None,
                None,
                None,
            );
            match super::copy_media_files(
                &src_basedir,
                &retry_context.dest_dir,
                files_to_copy.into_iter(),
                &options,
                error_tracker,
                progress_context.total_media_files,
                &mut progress_context.successful_media_files,
            ) {
                Ok((_, successful_files)) => {
                    let successful_count = successful_files.len();

                    // Update copied_files set
                    for file in successful_files {
                        media_context
                            .copied_files
                            .insert((src_basedir.clone(), file));
                    }

                    Ok((true, successful_count))
                }
                Err(e) => {
                    eprintln!("Error copying media files for playlist {}: {}", playlist, e);
                    if !options.keep_going {
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
            if !options.keep_going {
                return Err(e);
            }
            Ok((false, 0))
        }
    }
}

/// Retry copying a single media file from the error file
///
/// This function has been refactored to use:
/// 1. A MediaFileInfo struct instead of separate src_basedir and file parameters
/// 2. Grouped parameters for better organization using context structs
/// This reduces the number of arguments from the original 9 to 5.
pub fn retry_media_file(
    media_file: &MediaFileInfo,
    retry_context: &RetryContext,
    options: &super::CommandOptions,
    error_tracker: &mut Option<&mut super::ErrorTracker>,
    media_context: &mut MediaContext,
    progress_context: &mut ProgressContext,
) -> Result<usize> {
    let file_full_path = Path::new(&media_file.src_basedir).join(&media_file.file);

    super::print_message(
        options.verbose,
        "Retrying media file \"{}\"",
        &[&file_full_path.to_string_lossy()],
        None,
        None,
        None,
    );

    // Check if this file has already been copied
    if media_context
        .copied_files
        .contains(&(media_file.src_basedir.clone(), media_file.file.clone()))
    {
        super::print_message(
            options.verbose,
            "Skipping already copied file \"{}\"",
            &[&file_full_path.to_string_lossy()],
            None,
            None,
            None,
        );
        return Ok(1);
    }

    // Copy the file
    match super::copy_media_files(
        &media_file.src_basedir,
        &retry_context.dest_dir,
        std::iter::once(media_file.file.clone()),
        options,
        error_tracker,
        progress_context.total_media_files,
        &mut progress_context.successful_media_files,
    ) {
        Ok((_, successful_files)) => {
            let successful_count = successful_files.len();

            // Update copied_files set
            for file in successful_files {
                media_context
                    .copied_files
                    .insert((media_file.src_basedir.clone(), file));
            }

            Ok(successful_count)
        }
        Err(e) => {
            eprintln!(
                "Error copying media file {}: {}",
                file_full_path.display(),
                e
            );
            if !options.keep_going {
                return Err(e);
            }
            Ok(0)
        }
    }
}

/// Process retry operations from an error file
pub fn retry_operations(
    retry_file: &str,
    dest_dir: &str,
    options: &super::CommandOptions,
    error_tracker: &mut Option<&mut super::ErrorTracker>,
) -> Result<(usize, usize, usize, usize)> {
    super::print_message(
        options.verbose,
        "Retrying operations from error file \"{}\"",
        &[retry_file],
        None,
        None,
        None,
    );

    let (playlists, media_files) = parse_error_file(retry_file)?;

    let total_playlists = playlists.len();
    let total_media_files = media_files.len();
    let mut successful_playlists = 0;
    let mut successful_media_files = 0;

    // Create context structs
    let retry_context = RetryContext {
        dest_dir: dest_dir.to_string(),
    };

    let mut media_context = MediaContext {
        media_files_map: Vec::new(),
        copied_files: HashSet::new(),
    };

    let mut progress_context = ProgressContext {
        current_playlist_num: None,
        total_playlists: Some(total_playlists),
        total_media_files: Some(total_media_files),
        successful_media_files: 0,
    };

    // Process playlists first
    for (i, playlist) in playlists.iter().enumerate() {
        progress_context.current_playlist_num = Some(i + 1);

        match retry_playlist(
            playlist,
            &retry_context,
            options,
            error_tracker,
            &mut media_context,
            &mut progress_context,
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
    for (_i, (src_basedir, file)) in media_files.iter().enumerate() {
        let media_file = MediaFileInfo {
            src_basedir: src_basedir.clone(),
            file: file.clone(),
        };

        match retry_media_file(
            &media_file,
            &retry_context,
            options,
            error_tracker,
            &mut media_context,
            &mut progress_context,
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
