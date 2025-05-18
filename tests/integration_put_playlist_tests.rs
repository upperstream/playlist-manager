use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

mod integration_test_common;

#[cfg(test)]
mod tests {
    use super::*;
    use integration_test_common::{create_test_file, setup_test_directory};

    // Helper function to verify file exists and has expected content
    fn verify_file(path: &Path, expected_content: &str) -> bool {
        if !path.exists() {
            return false;
        }

        let content = std::fs::read_to_string(path).unwrap();
        content == expected_content
    }

    #[test]
    fn test_put_playlist_basic() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        let playlist_path = music_dir.join("playlist.m3u8");

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        assert
            .success()
            .stdout(predicate::str::contains("(1/1) playlist copied"))
            .stdout(predicate::str::contains("media files copied"));

        // Verify playlist was copied
        assert!(dest_dir.join("playlist.m3u8").exists());

        // Verify media files were copied
        assert!(dest_dir.join("artist1/album1/title1.flac").exists());
        assert!(dest_dir.join("artist1/album1/title2.flac").exists());
        assert!(dest_dir.join("artist2/album1/title1.flac").exists());
        assert!(dest_dir.join("artist2/album2/title1.flac").exists());

        // Verify content of files
        assert!(verify_file(
            &dest_dir.join("artist1/album1/title1.flac"),
            "test content 1"
        ));
        assert!(verify_file(
            &dest_dir.join("artist1/album1/title2.flac"),
            "test content 2"
        ));
        assert!(verify_file(
            &dest_dir.join("artist2/album1/title1.flac"),
            "test content 3"
        ));
        assert!(verify_file(
            &dest_dir.join("artist2/album2/title1.flac"),
            "test content 4"
        ));
    }

    #[test]
    fn test_put_playlist_with_backslashes() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create playlist with backslashes
        let playlist_content =
            "artist1\\album1\\title1.flac\nartist1\\album1\\title2.flac\nartist2\\album1\\title1.flac\nartist2\\album2\\title1.flac";
        let playlist_path = music_dir.join("playlist_backslash.m3u8");
        create_test_file(&playlist_path, playlist_content);

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        assert
            .success()
            .stdout(predicate::str::contains("(1/1) playlist copied"))
            .stdout(predicate::str::contains("media files copied"));

        // Verify playlist was copied and backslashes were replaced
        let dest_playlist = dest_dir.join("playlist_backslash.m3u8");
        assert!(dest_playlist.exists());

        let content = fs::read_to_string(dest_playlist).unwrap();
        assert!(content.contains("artist1/album1/title1.flac"));
        assert!(!content.contains("artist1\\album1\\title1.flac"));

        // Verify media files were copied
        assert!(dest_dir.join("artist1/album1/title1.flac").exists());
        assert!(dest_dir.join("artist1/album1/title2.flac").exists());
        assert!(dest_dir.join("artist2/album1/title1.flac").exists());
        assert!(dest_dir.join("artist2/album2/title1.flac").exists());
    }

    #[test]
    fn test_put_playlist_verbose() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        let playlist_path = music_dir.join("playlist.m3u8");

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("-v")
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        assert
            .success()
            .stdout(predicate::str::contains("(1/1) playlist copied"))
            .stdout(predicate::str::contains("media files copied"))
            .stderr(predicate::str::contains("Copy playlist"));

        // Note: No error messages should be present for missing lyrics files
        // even in verbose mode
    }

    #[test]
    fn test_put_playlist_multiple() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create a second playlist
        let playlist2_content = "artist1/album1/title1.flac\nartist2/album2/title1.flac";
        let playlist2_path = music_dir.join("playlist2.m3u8");
        create_test_file(&playlist2_path, playlist2_content);

        let playlist1_path = music_dir.join("playlist.m3u8");

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist1_path.to_str().unwrap())
            .arg(playlist2_path.to_str().unwrap())
            .assert();

        assert
            .success()
            .stdout(predicate::str::contains("(2/2) playlist copied"))
            .stdout(predicate::str::contains("media files copied"));

        // Verify both playlists were copied
        assert!(dest_dir.join("playlist.m3u8").exists());
        assert!(dest_dir.join("playlist2.m3u8").exists());
    }

    #[test]
    fn test_put_playlist_invalid_dest() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");

        // Use a file as destination instead of a directory
        let invalid_dest = music_dir.join("artist1/album1/title1.flac");
        let playlist_path = music_dir.join("playlist.m3u8");

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg(invalid_dest.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        assert.failure().code(255);
    }

    #[test]
    fn test_put_playlist_missing_args() {
        let temp_dir = setup_test_directory();
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Missing playlist argument
        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd.arg(dest_dir.to_str().unwrap()).assert();

        assert.failure();
    }

    #[test]
    fn test_put_playlist_with_lyrics() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        let playlist_path = music_dir.join("playlist.m3u8");

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("--lyrics")
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        // Note: No error messages are expected when lyrics files are not found
        assert
            .success()
            .stdout(predicate::str::contains("(1/1) playlist copied"));

        // Verify media files were copied
        assert!(dest_dir.join("artist1/album1/title1.flac").exists());
        assert!(dest_dir.join("artist1/album1/title2.flac").exists());
        assert!(dest_dir.join("artist2/album1/title1.flac").exists());
        assert!(dest_dir.join("artist2/album2/title1.flac").exists());

        // Verify lyrics files were copied
        assert!(dest_dir.join("artist1/album1/title1.lrc").exists());
        assert!(dest_dir.join("artist2/album2/title1.lrc").exists());

        // Verify lyrics files have correct content
        assert!(verify_file(
            &dest_dir.join("artist1/album1/title1.lrc"),
            "[00:00.00] Lyrics for title1"
        ));
        assert!(verify_file(
            &dest_dir.join("artist2/album2/title1.lrc"),
            "[00:00.00] Lyrics for another title1"
        ));

        // Verify lyrics files don't exist for files that didn't have them
        // (and no error messages are generated for these missing files)
        assert!(!dest_dir.join("artist1/album1/title2.lrc").exists());
        assert!(!dest_dir.join("artist2/album1/title1.lrc").exists());
    }

    #[test]
    fn test_put_playlist_with_lyrics_none_found() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create a playlist with files that don't have lyrics
        let playlist_content = "artist1/album1/title2.flac\nartist2/album1/title1.flac";
        let playlist_path = music_dir.join("playlist_no_lyrics.m3u8");
        create_test_file(&playlist_path, playlist_content);

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("--lyrics")
            .arg("-v") // Use verbose mode to ensure we would see any error messages
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        // Command should succeed without error messages about missing lyrics files
        assert
            .success()
            .stdout(predicate::str::contains("(1/1) playlist copied"))
            .stdout(predicate::str::contains("media files copied"));

        // Verify media files were copied
        assert!(dest_dir.join("artist1/album1/title2.flac").exists());
        assert!(dest_dir.join("artist2/album1/title1.flac").exists());

        // Verify no lyrics files were copied (as they don't exist)
        assert!(!dest_dir.join("artist1/album1/title2.lrc").exists());
        assert!(!dest_dir.join("artist2/album1/title1.lrc").exists());
    }

    #[test]
    fn test_put_playlist_keep_going_output_format() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        let playlist_path = music_dir.join("playlist.m3u8");

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("--keep-going")
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        // Verify the output format with (a/b) statistics
        assert
            .success()
            .stdout(predicate::str::contains("(1/1) playlist copied"))
            .stdout(predicate::str::contains("media files copied"));
    }

    #[test]
    fn test_put_playlist_keep_going_with_missing_playlist() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        let existing_playlist = music_dir.join("playlist.m3u8");
        let missing_playlist = music_dir.join("missing.m3u8");

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("--keep-going")
            .arg(dest_dir.to_str().unwrap())
            .arg(existing_playlist.to_str().unwrap())
            .arg(missing_playlist.to_str().unwrap())
            .assert();

        // Command should succeed with --keep-going despite the missing playlist
        assert
            .success()
            .stdout(predicate::str::contains("(1/2) playlist copied"))
            .stdout(predicate::str::contains("media files copied"));

        // Verify the existing playlist was copied
        assert!(dest_dir.join("playlist.m3u8").exists());
    }

    #[test]
    fn test_put_playlist_keep_going_with_missing_media_file() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create a playlist with a missing file
        let playlist_content =
            "artist1/album1/title1.flac\nartist1/album1/missing.flac\nartist2/album1/title1.flac";
        let playlist_path = music_dir.join("playlist_with_missing.m3u8");
        create_test_file(&playlist_path, playlist_content);

        // Create a second playlist without missing files
        let playlist2_content = "artist1/album1/title2.flac\nartist2/album2/title1.flac";
        let playlist2_path = music_dir.join("playlist2.m3u8");
        create_test_file(&playlist2_path, playlist2_content);

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("--keep-going")
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .arg(playlist2_path.to_str().unwrap())
            .assert();

        // Command should succeed with --keep-going despite the missing media file
        assert
            .success()
            .stdout(predicate::str::contains("(2/2) playlist copied"));

        // Verify both playlists were copied (even though one has missing files)
        assert!(dest_dir.join("playlist_with_missing.m3u8").exists());
        assert!(dest_dir.join("playlist2.m3u8").exists());

        // Verify the files from the second playlist were copied
        assert!(dest_dir.join("artist1/album1/title2.flac").exists());
        assert!(dest_dir.join("artist2/album2/title1.flac").exists());
    }

    #[test]
    fn test_put_playlist_without_keep_going_fails_on_missing_playlist() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        let existing_playlist = music_dir.join("playlist.m3u8");
        let missing_playlist = music_dir.join("missing.m3u8");

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg(dest_dir.to_str().unwrap())
            .arg(existing_playlist.to_str().unwrap())
            .arg(missing_playlist.to_str().unwrap())
            .assert();

        // Command should fail without --keep-going when a playlist is missing
        assert.failure();
    }

    #[test]
    fn test_error_files_without_keep_going() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");
        let error_file = temp_dir.path().join("errors.log");

        fs::create_dir_all(&dest_dir).unwrap();

        let playlist_path = music_dir.join("playlist.m3u8");

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("--error-files")
            .arg(error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        // Command should fail with exit code 255 when --error-files is used without --keep-going
        assert.failure().code(255).stderr(predicate::str::contains(
            "--error-files can only be used with --keep-going",
        ));
    }

    #[test]
    fn test_error_files_with_keep_going() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");
        let error_file = temp_dir.path().join("errors.log");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create a playlist with a missing file
        let playlist_content =
            "artist1/album1/title1.flac\nartist1/album1/missing.flac\nartist2/album1/title1.flac";
        let playlist_path = music_dir.join("playlist_with_missing.m3u8");
        create_test_file(&playlist_path, playlist_content);

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("--keep-going")
            .arg("--error-files")
            .arg(error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        // Command should succeed with --keep-going and --error-files
        assert.success();

        // Verify error log file exists and contains the missing file with correct prefix
        assert!(error_file.exists());
        let error_content = fs::read_to_string(&error_file).unwrap();
        assert!(error_content.contains("M "));
        assert!(error_content.contains("artist1/album1/missing.flac"));
    }

    #[test]
    fn test_error_files_with_multiple_errors() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");
        let error_file = temp_dir.path().join("errors.log");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create a playlist with multiple missing files
        let playlist1_content =
            "artist1/album1/title1.flac\nartist1/album1/missing1.flac\nartist2/album1/title1.flac";
        let playlist1_path = music_dir.join("playlist_with_missing1.m3u8");
        create_test_file(&playlist1_path, playlist1_content);

        // Create a second playlist with a missing file
        let playlist2_content = "artist1/album1/title2.flac\nartist2/album2/missing2.flac";
        let playlist2_path = music_dir.join("playlist_with_missing2.m3u8");
        create_test_file(&playlist2_path, playlist2_content);

        // Create a third playlist that doesn't exist
        let missing_playlist_path = music_dir.join("missing_playlist.m3u8");

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("--keep-going")
            .arg("--error-files")
            .arg(error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist1_path.to_str().unwrap())
            .arg(playlist2_path.to_str().unwrap())
            .arg(missing_playlist_path.to_str().unwrap())
            .assert();

        // Command should succeed with --keep-going and --error-files
        assert.success();

        // Verify error log file exists and contains all the missing files and playlists with correct prefixes
        assert!(error_file.exists());
        let error_content = fs::read_to_string(&error_file).unwrap();

        // Check for playlist prefix
        assert!(error_content.contains("P "));
        assert!(error_content.contains(&format!("P {}", missing_playlist_path.to_str().unwrap())));

        // Check for media file prefixes
        assert!(error_content.contains("M "));
        assert!(error_content.contains("artist1/album1/missing1.flac"));
        assert!(error_content.contains("artist2/album2/missing2.flac"));
    }

    #[test]
    fn test_error_files_format() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");
        let error_file = temp_dir.path().join("errors.log");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create a playlist that will fail (invalid path)
        let missing_playlist_path = music_dir.join("missing_playlist.m3u8");

        // Create a playlist with a missing file
        let playlist_content =
            "artist1/album1/title1.flac\nartist1/album1/missing.flac\nartist2/album1/title1.flac";
        let playlist_path = music_dir.join("playlist_with_missing.m3u8");
        create_test_file(&playlist_path, playlist_content);

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("--keep-going")
            .arg("--error-files")
            .arg(error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .arg(missing_playlist_path.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        // Command should succeed with --keep-going and --error-files
        assert.success();

        // Verify error log file exists
        assert!(error_file.exists());
        let error_content = fs::read_to_string(&error_file).unwrap();

        // The first line should be the failed playlist with P prefix
        let lines: Vec<&str> = error_content.lines().collect();
        assert!(!lines.is_empty());
        assert!(lines[0].starts_with("P "));
        assert!(lines[0].contains(missing_playlist_path.to_str().unwrap()));

        // The subsequent lines should be the failed media files with M prefix
        let media_lines: Vec<&str> = lines
            .iter()
            .filter(|line| line.starts_with("M "))
            .cloned()
            .collect();
        assert!(!media_lines.is_empty());

        // Verify that media files from failed playlists are not included
        // (i.e., there should be no entries for files from missing_playlist.m3u8)
        for line in &lines {
            if line.starts_with("M ") {
                assert!(!line.contains(missing_playlist_path.to_str().unwrap()));
            }
        }
    }

    #[test]
    fn test_retry_basic() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");
        let error_file = temp_dir.path().join("errors.log");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create a playlist with a missing file
        let playlist_content =
            "artist1/album1/title1.flac\nartist1/album1/missing.flac\nartist2/album1/title1.flac";
        let playlist_path = music_dir.join("playlist_with_missing.m3u8");
        create_test_file(&playlist_path, playlist_content);

        // First run: create error file
        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("--keep-going")
            .arg("--error-files")
            .arg(error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        assert.success();
        assert!(error_file.exists());

        // Print the content of the error file for debugging
        let error_content = fs::read_to_string(&error_file).unwrap();
        println!("Error file content:\n{}", error_content);

        // Create the missing file before retry
        create_test_file(
            &music_dir.join("artist1/album1/missing.flac"),
            "test content for missing file",
        );

        // Clean destination directory
        fs::remove_dir_all(&dest_dir).unwrap();
        fs::create_dir_all(&dest_dir).unwrap();

        // Second run: retry with error file
        let mut retry_cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let retry_assert = retry_cmd
            .arg("--retry")
            .arg(error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .assert();

        retry_assert.success();

        // Verify the previously missing file was copied
        assert!(dest_dir.join("artist1/album1/missing.flac").exists());
        let content = fs::read_to_string(dest_dir.join("artist1/album1/missing.flac")).unwrap();
        assert_eq!(content, "test content for missing file");
    }

    #[test]
    fn test_retry_with_error_file() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");
        let error_file = temp_dir.path().join("errors.log");
        let new_error_file = temp_dir.path().join("new_errors.log");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create a playlist with two missing files
        let playlist_content =
            "artist1/album1/title1.flac\nartist1/album1/missing1.flac\nartist1/album1/missing2.flac";
        let playlist_path = music_dir.join("playlist_with_missing.m3u8");
        create_test_file(&playlist_path, playlist_content);

        // First run: create error file
        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("--keep-going")
            .arg("--error-files")
            .arg(error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        assert.success();
        assert!(error_file.exists());

        // Create only one of the missing files before retry
        create_test_file(
            &music_dir.join("artist1/album1/missing1.flac"),
            "test content for missing1 file",
        );

        // Clean destination directory
        fs::remove_dir_all(&dest_dir).unwrap();
        fs::create_dir_all(&dest_dir).unwrap();

        // Second run: retry with error file and create new error file
        let mut retry_cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let retry_assert = retry_cmd
            .arg("--retry")
            .arg(error_file.to_str().unwrap())
            .arg("--keep-going")
            .arg("--error-files")
            .arg(new_error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .assert();

        retry_assert.success();

        // Verify the first missing file was copied
        assert!(dest_dir.join("artist1/album1/missing1.flac").exists());
        let content = fs::read_to_string(dest_dir.join("artist1/album1/missing1.flac")).unwrap();
        assert_eq!(content, "test content for missing1 file");

        // Verify the second missing file is still missing and in the new error file
        assert!(!dest_dir.join("artist1/album1/missing2.flac").exists());
        assert!(new_error_file.exists());
        let error_content = fs::read_to_string(&new_error_file).unwrap();
        assert!(error_content.contains("missing2.flac"));
    }

    #[test]
    fn test_retry_with_lyrics() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");
        let error_file = temp_dir.path().join("errors.log");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create an error file with media entries
        let error_content = format!(
            "M {}/artist1/album1/title1.flac",
            music_dir.to_str().unwrap()
        );
        create_test_file(&error_file, &error_content);

        // Run retry with lyrics option
        let mut retry_cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let retry_assert = retry_cmd
            .arg("--retry")
            .arg(error_file.to_str().unwrap())
            .arg("--lyrics")
            .arg(dest_dir.to_str().unwrap())
            .assert();

        retry_assert.success();

        // Verify media file was copied
        assert!(dest_dir.join("artist1/album1/title1.flac").exists());

        // Verify lyrics file was also copied
        assert!(dest_dir.join("artist1/album1/title1.lrc").exists());

        // Verify lyrics file has correct content
        let lyrics_content =
            fs::read_to_string(dest_dir.join("artist1/album1/title1.lrc")).unwrap();
        assert_eq!(lyrics_content, "[00:00.00] Lyrics for title1");
    }

    #[test]
    fn test_retry_same_error_file() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");
        let error_file = temp_dir.path().join("errors.log");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create a playlist with a missing file
        let playlist_content = "artist1/album1/title1.flac\nartist1/album1/missing.flac";
        let playlist_path = music_dir.join("playlist_with_missing.m3u8");
        create_test_file(&playlist_path, playlist_content);

        // First run: create error file
        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        cmd.arg("--keep-going")
            .arg("--error-files")
            .arg(error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert()
            .success();

        // Second run: try to use same file for retry and error-files
        let mut retry_cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let retry_assert = retry_cmd
            .arg("--retry")
            .arg(error_file.to_str().unwrap())
            .arg("--keep-going")
            .arg("--error-files")
            .arg(error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .assert();

        // Should fail with exit code 255
        retry_assert
            .failure()
            .code(255)
            .stderr(predicate::str::contains("cannot specify the same file"));
    }

    #[test]
    fn test_retry_playlist_and_media() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create an error file with both playlist and media entries
        let error_file = temp_dir.path().join("errors.log");
        let error_content = format!(
            "P {}\nM {}/artist1/album1/missing.flac",
            music_dir.join("playlist.m3u8").to_str().unwrap(),
            music_dir.to_str().unwrap()
        );
        create_test_file(&error_file, &error_content);

        // Create the missing file
        create_test_file(
            &music_dir.join("artist1/album1/missing.flac"),
            "test content for missing file",
        );

        // Run retry
        let mut retry_cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let retry_assert = retry_cmd
            .arg("--retry")
            .arg(error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .assert();

        retry_assert.success();

        // Verify both playlist and media file were copied
        assert!(dest_dir.join("playlist.m3u8").exists());
        assert!(dest_dir.join("artist1/album1/missing.flac").exists());
    }

    #[test]
    fn test_retry_consecutive_playlists() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create a second playlist
        let playlist2_content = "artist1/album1/title2.flac\nartist2/album2/title1.flac";
        let playlist2_path = music_dir.join("playlist2.m3u8");
        create_test_file(&playlist2_path, playlist2_content);

        // Create an error file with consecutive playlist entries
        let error_file = temp_dir.path().join("errors.log");
        let error_content = format!(
            "P {}\nP {}",
            music_dir.join("playlist.m3u8").to_str().unwrap(),
            playlist2_path.to_str().unwrap()
        );
        create_test_file(&error_file, &error_content);

        // Run retry
        let mut retry_cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let retry_assert = retry_cmd
            .arg("--retry")
            .arg(error_file.to_str().unwrap())
            .arg(dest_dir.to_str().unwrap())
            .assert();

        retry_assert.success();

        // Verify both playlists were copied
        assert!(dest_dir.join("playlist.m3u8").exists());
        assert!(dest_dir.join("playlist2.m3u8").exists());

        // Verify media files from both playlists were copied
        assert!(dest_dir.join("artist1/album1/title1.flac").exists());
        assert!(dest_dir.join("artist1/album1/title2.flac").exists());
        assert!(dest_dir.join("artist2/album1/title1.flac").exists());
        assert!(dest_dir.join("artist2/album2/title1.flac").exists());
    }

    // Helper function to extract file numbers from verbose output
    fn extract_file_numbers(output: &str) -> Vec<usize> {
        let mut numbers = Vec::new();

        // Regular expression to match patterns like "(1-M/4)", "(2-M/4)", etc.
        let re = regex::Regex::new(r"\((\d+)(?:-[ML])?/\d+\)").unwrap();

        for line in output.lines() {
            if line.contains("Copy track") {
                if let Some(captures) = re.captures(line) {
                    if let Some(number_str) = captures.get(1) {
                        if let Ok(number) = number_str.as_str().parse::<usize>() {
                            numbers.push(number);
                        }
                    }
                }
            }
        }

        numbers
    }

    #[test]
    fn test_file_counting_across_playlists() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create two playlists with distinct files
        let playlist1_content = "artist1/album1/title1.flac\nartist1/album1/title2.flac";
        let playlist1_path = music_dir.join("playlist1.m3u8");
        create_test_file(&playlist1_path, playlist1_content);

        let playlist2_content = "artist2/album1/title1.flac\nartist2/album2/title1.flac";
        let playlist2_path = music_dir.join("playlist2.m3u8");
        create_test_file(&playlist2_path, playlist2_content);

        // Run with verbose mode to capture progress messages
        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("-v")
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist1_path.to_str().unwrap())
            .arg(playlist2_path.to_str().unwrap())
            .assert();

        // Capture stderr output which contains the progress messages
        let output = String::from_utf8_lossy(&assert.get_output().stderr);

        // Extract file numbers from the output
        let file_numbers = extract_file_numbers(&output);

        // Verify that file numbers are sequential across playlists
        // The fixed implementation numbers files as [1, 2, 3, 4]
        assert_eq!(file_numbers, vec![1, 2, 3, 4]);

        // Verify all files were copied
        assert!(dest_dir.join("artist1/album1/title1.flac").exists());
        assert!(dest_dir.join("artist1/album1/title2.flac").exists());
        assert!(dest_dir.join("artist2/album1/title1.flac").exists());
        assert!(dest_dir.join("artist2/album2/title1.flac").exists());
    }

    #[test]
    fn test_only_successful_files_counted() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create a playlist with some files that will fail to copy
        let playlist_content =
            "artist1/album1/title1.flac\nartist1/album1/missing.flac\nartist2/album1/title1.flac";
        let playlist_path = music_dir.join("playlist_with_missing.m3u8");
        create_test_file(&playlist_path, playlist_content);

        // Run with verbose and keep-going mode
        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("-v")
            .arg("--keep-going")
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        // Capture stderr output which contains the progress messages
        let output = String::from_utf8_lossy(&assert.get_output().stderr);

        // Extract file numbers from the output
        let file_numbers = extract_file_numbers(&output);

        // Verify that only successful files are counted
        // We expect 2 files numbered 1, 2 (the missing file is skipped)
        assert_eq!(file_numbers, vec![1, 2]);

        // Verify successful files were copied
        assert!(dest_dir.join("artist1/album1/title1.flac").exists());
        assert!(dest_dir.join("artist2/album1/title1.flac").exists());

        // Verify missing file was not copied
        assert!(!dest_dir.join("artist1/album1/missing.flac").exists());
    }

    #[test]
    fn test_counting_with_shared_files() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create two playlists with some shared files
        let playlist1_content =
            "artist1/album1/title1.flac\nartist1/album1/title2.flac\nartist2/album1/title1.flac";
        let playlist1_path = music_dir.join("playlist1.m3u8");
        create_test_file(&playlist1_path, playlist1_content);

        let playlist2_content =
            "artist1/album1/title2.flac\nartist2/album1/title1.flac\nartist2/album2/title1.flac";
        let playlist2_path = music_dir.join("playlist2.m3u8");
        create_test_file(&playlist2_path, playlist2_content);

        // Run with verbose mode
        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("-v")
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist1_path.to_str().unwrap())
            .arg(playlist2_path.to_str().unwrap())
            .assert();

        // Capture stderr output which contains the progress messages
        let output = String::from_utf8_lossy(&assert.get_output().stderr);

        // Extract file numbers from the output
        let file_numbers = extract_file_numbers(&output);

        // Verify that shared files are only counted once
        // The fixed implementation numbers files as [1, 2, 3, 4]
        assert_eq!(file_numbers, vec![1, 2, 3, 4]);

        // Verify all files were copied
        assert!(dest_dir.join("artist1/album1/title1.flac").exists());
        assert!(dest_dir.join("artist1/album1/title2.flac").exists());
        assert!(dest_dir.join("artist2/album1/title1.flac").exists());
        assert!(dest_dir.join("artist2/album2/title1.flac").exists());
    }

    #[test]
    fn test_summary_count_matches_verbose_count() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        let playlist_path = music_dir.join("playlist.m3u8");

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let output = cmd
            .arg("-v")
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Extract the count from summary output
        let summary_count_regex = regex::Regex::new(r"\((\d+)/\d+\) media files copied").unwrap();
        let summary_count = summary_count_regex
            .captures(&stdout)
            .expect("Failed to find media files count in summary")
            .get(1)
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap();

        // Count "Copy track" messages in verbose output
        let verbose_count = stderr
            .lines()
            .filter(|line| line.contains("Copy track"))
            .count();

        // The counts should match
        assert_eq!(
            verbose_count, summary_count,
            "Summary count ({}) does not match verbose output count ({})",
            summary_count, verbose_count
        );
    }

    #[test]
    fn test_total_count_consistent_across_playlists() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create two playlists with distinct files
        let playlist1_content = "artist1/album1/title1.flac\nartist1/album1/title2.flac";
        let playlist1_path = music_dir.join("playlist1.m3u8");
        create_test_file(&playlist1_path, playlist1_content);

        let playlist2_content = "artist2/album1/title1.flac\nartist2/album2/title1.flac";
        let playlist2_path = music_dir.join("playlist2.m3u8");
        create_test_file(&playlist2_path, playlist2_content);

        // Run with verbose mode to capture progress messages
        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("-v")
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist1_path.to_str().unwrap())
            .arg(playlist2_path.to_str().unwrap())
            .assert();

        // Capture stderr output which contains the progress messages
        let output = String::from_utf8_lossy(&assert.get_output().stderr);

        // Extract total counts from each playlist's media file messages
        let re = regex::Regex::new(r"\(\d+(?:-[ML])?/(\d+)\).*Copy track").unwrap();
        let mut total_counts = Vec::new();

        for line in output.lines() {
            if line.contains("Copy track") {
                if let Some(captures) = re.captures(line) {
                    if let Some(total_str) = captures.get(1) {
                        if let Ok(total) = total_str.as_str().parse::<usize>() {
                            total_counts.push(total);
                        }
                    }
                }
            }
        }

        // Verify we have at least one count from each playlist
        assert!(!total_counts.is_empty(), "No total counts found in output");

        // Get the expected total count (4 unique files across both playlists)
        let expected_total = 4;

        // Verify all total counts are equal to the expected total
        for (i, &count) in total_counts.iter().enumerate() {
            assert_eq!(
                count,
                expected_total,
                "Total count in message {} is {}, expected {}",
                i + 1,
                count,
                expected_total
            );
        }

        // Verify all files were copied
        assert!(dest_dir.join("artist1/album1/title1.flac").exists());
        assert!(dest_dir.join("artist1/album1/title2.flac").exists());
        assert!(dest_dir.join("artist2/album1/title1.flac").exists());
        assert!(dest_dir.join("artist2/album2/title1.flac").exists());
    }

    #[test]
    fn test_counting_with_failed_files_and_multiple_playlists() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let dest_dir = temp_dir.path().join("DEST");

        fs::create_dir_all(&dest_dir).unwrap();

        // Create playlists with some shared files and some that will fail
        let playlist1_content =
            "artist1/album1/title1.flac\nartist1/album1/missing1.flac\nartist2/album1/title1.flac";
        let playlist1_path = music_dir.join("playlist1.m3u8");
        create_test_file(&playlist1_path, playlist1_content);

        let playlist2_content =
            "artist1/album1/title2.flac\nartist2/album1/title1.flac\nartist1/album1/missing2.flac";
        let playlist2_path = music_dir.join("playlist2.m3u8");
        create_test_file(&playlist2_path, playlist2_content);

        // Run with verbose and keep-going mode
        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg("-v")
            .arg("--keep-going")
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist1_path.to_str().unwrap())
            .arg(playlist2_path.to_str().unwrap())
            .assert();

        // Capture stderr output which contains the progress messages
        let output = String::from_utf8_lossy(&assert.get_output().stderr);

        // Extract file numbers from the output
        let file_numbers = extract_file_numbers(&output);

        // Verify that:
        // 1. Failed files are skipped in the count
        // 2. Shared files are only counted once
        // 3. The counter is continuous across playlists
        // The fixed implementation numbers files as [1, 2, 3]
        assert_eq!(file_numbers, vec![1, 2, 3]);

        // Verify successful files were copied
        assert!(dest_dir.join("artist1/album1/title1.flac").exists());
        assert!(dest_dir.join("artist1/album1/title2.flac").exists());
        assert!(dest_dir.join("artist2/album1/title1.flac").exists());

        // Verify missing files were not copied
        assert!(!dest_dir.join("artist1/album1/missing1.flac").exists());
        assert!(!dest_dir.join("artist1/album1/missing2.flac").exists());
    }
}
