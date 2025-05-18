use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};

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
    dest_dir: &str,
    options: &super::CommandOptions,
    error_tracker: &mut Option<&mut super::ErrorTracker>,
    media_files_map: &mut Vec<(String, HashSet<String>)>,
    copied_files: &mut HashSet<(String, String)>,
    current_playlist_num: Option<usize>,
    total_playlists: Option<usize>,
    total_media_files: Option<usize>,
    successful_media_files: &mut usize,
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
        dest_dir,
        options.verbose,
        media_files_map,
        current_playlist_num,
        total_playlists,
    ) {
        Ok((src_basedir, files)) => {
            // Copy media files for this playlist
            let files_to_copy =
                super::filter_already_copied_files(&src_basedir, &files, copied_files);

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
                dest_dir,
                files_to_copy.into_iter(),
                &options,
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
pub fn retry_media_file(
    src_basedir: &str,
    file: &str,
    dest_dir: &str,
    options: &super::CommandOptions,
    error_tracker: &mut Option<&mut super::ErrorTracker>,
    copied_files: &mut HashSet<(String, String)>,
    _current_file_num: Option<usize>,
    total_media_files: Option<usize>,
    successful_media_files: &mut usize,
) -> Result<usize> {
    super::print_message(
        options.verbose,
        "Retrying media file \"{}\"",
        &[&Path::new(src_basedir).join(file).to_string_lossy()],
        None,
        None,
        None,
    );

    // Check if this file has already been copied
    if copied_files.contains(&(src_basedir.to_string(), file.to_string())) {
        super::print_message(
            options.verbose,
            "Skipping already copied file \"{}\"",
            &[&Path::new(src_basedir).join(file).to_string_lossy()],
            None,
            None,
            None,
        );
        return Ok(1);
    }

    // Copy the file
    match super::copy_media_files(
        src_basedir,
        dest_dir,
        std::iter::once(file.to_string()),
        &options,
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
    let mut media_files_map: Vec<(String, HashSet<String>)> = Vec::new();
    let mut copied_files: HashSet<(String, String)> = HashSet::new();

    // Process playlists first
    for (i, playlist) in playlists.iter().enumerate() {
        match retry_playlist(
            playlist,
            dest_dir,
            &options,
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
            &options,
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
