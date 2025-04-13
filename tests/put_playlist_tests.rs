use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use common::{create_test_file, setup_test_directory};

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
            .stdout(predicate::str::contains("Number of copied playlists: 1"))
            .stdout(predicate::str::contains("Number of copied media files: 4"));

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
        let playlist_content = "artist1\\album1\\title1.flac\nartist1\\album1\\title2.flac\nartist2\\album1\\title1.flac\nartist2\\album2\\title1.flac";
        let playlist_path = music_dir.join("playlist_backslash.m3u8");
        create_test_file(&playlist_path, playlist_content);

        let mut cmd = Command::cargo_bin("plm-put-playlist").unwrap();
        let assert = cmd
            .arg(dest_dir.to_str().unwrap())
            .arg(playlist_path.to_str().unwrap())
            .assert();

        assert
            .success()
            .stdout(predicate::str::contains("Number of copied playlists: 1"))
            .stdout(predicate::str::contains("Number of copied media files: 4"));

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
            .stdout(predicate::str::contains("Number of copied playlists: 1"))
            .stdout(predicate::str::contains("Number of copied media files: 4"))
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
            .stdout(predicate::str::contains("Number of copied playlists: 2"))
            .stdout(predicate::str::contains("Number of copied media files: 4"));

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
            .stdout(predicate::str::contains("Number of copied playlists: 1"));

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
            .stdout(predicate::str::contains("Number of copied playlists: 1"))
            .stdout(predicate::str::contains("Number of copied media files: 2"));

        // Verify media files were copied
        assert!(dest_dir.join("artist1/album1/title2.flac").exists());
        assert!(dest_dir.join("artist2/album1/title1.flac").exists());

        // Verify no lyrics files were copied (as they don't exist)
        assert!(!dest_dir.join("artist1/album1/title2.lrc").exists());
        assert!(!dest_dir.join("artist2/album1/title1.lrc").exists());
    }
}
