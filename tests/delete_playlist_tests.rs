use assert_cmd::Command;
use predicates::prelude::*;

mod common;
use common::{create_test_file, setup_test_directory};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_playlist_basic() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let playlist_path = music_dir.join("playlist.m3u8");

        // Verify playlist exists before deletion
        assert!(playlist_path.exists());

        let mut cmd = Command::cargo_bin("plm-delete-playlist").unwrap();
        let assert = cmd
            .arg(playlist_path.to_str().unwrap())
            .assert();

        assert.success();

        // Verify playlist was deleted
        assert!(!playlist_path.exists());

        // Verify media files still exist
        assert!(music_dir.join("artist1/album1/title1.flac").exists());
        assert!(music_dir.join("artist1/album1/title2.flac").exists());
        assert!(music_dir.join("artist2/album1/title1.flac").exists());
        assert!(music_dir.join("artist2/album2/title1.flac").exists());
    }

    #[test]
    fn test_delete_playlist_with_media() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let playlist_path = music_dir.join("playlist.m3u8");

        // Verify files exist before deletion
        assert!(playlist_path.exists());
        assert!(music_dir.join("artist1/album1/title1.flac").exists());
        assert!(music_dir.join("artist1/album1/title1.lrc").exists());
        assert!(music_dir.join("artist1/album1/title2.flac").exists());
        assert!(music_dir.join("artist2/album1/title1.flac").exists());
        assert!(music_dir.join("artist2/album2/title1.flac").exists());
        assert!(music_dir.join("artist2/album2/title1.lrc").exists());

        let mut cmd = Command::cargo_bin("plm-delete-playlist").unwrap();
        let assert = cmd
            .arg("--media")
            .arg(playlist_path.to_str().unwrap())
            .assert();

        assert.success();

        // Verify playlist was deleted
        assert!(!playlist_path.exists());

        // Verify media files were deleted
        assert!(!music_dir.join("artist1/album1/title1.flac").exists());
        assert!(!music_dir.join("artist1/album1/title1.lrc").exists());
        assert!(!music_dir.join("artist1/album1/title2.flac").exists());
        assert!(!music_dir.join("artist2/album1/title1.flac").exists());
        assert!(!music_dir.join("artist2/album2/title1.flac").exists());
        assert!(!music_dir.join("artist2/album2/title1.lrc").exists());

        // Verify empty directories were deleted
        assert!(!music_dir.join("artist1/album1").exists());
        assert!(!music_dir.join("artist1").exists());
        assert!(!music_dir.join("artist2/album1").exists());
        assert!(!music_dir.join("artist2/album2").exists());
        assert!(!music_dir.join("artist2").exists());
    }

    #[test]
    fn test_delete_playlist_verbose() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");
        let playlist_path = music_dir.join("playlist.m3u8");

        let mut cmd = Command::cargo_bin("plm-delete-playlist").unwrap();
        let assert = cmd
            .arg("-v")
            .arg(playlist_path.to_str().unwrap())
            .assert();

        assert
            .success()
            .stderr(predicate::str::contains("Deleting playlist"));
    }

    #[test]
    fn test_delete_playlist_multiple() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");

        // Create a second playlist
        let playlist2_content = "artist1/album1/title1.flac\nartist2/album2/title1.flac";
        let playlist2_path = music_dir.join("playlist2.m3u8");
        create_test_file(&playlist2_path, playlist2_content);

        let playlist1_path = music_dir.join("playlist.m3u8");

        // Verify playlists exist before deletion
        assert!(playlist1_path.exists());
        assert!(playlist2_path.exists());

        let mut cmd = Command::cargo_bin("plm-delete-playlist").unwrap();
        let assert = cmd
            .arg(playlist1_path.to_str().unwrap())
            .arg(playlist2_path.to_str().unwrap())
            .assert();

        assert.success();

        // Verify both playlists were deleted
        assert!(!playlist1_path.exists());
        assert!(!playlist2_path.exists());
    }

    #[test]
    fn test_delete_playlist_with_media_multiple() {
        let temp_dir = setup_test_directory();
        let music_dir = temp_dir.path().join("MUSIC");

        // Create a second playlist with some overlapping files
        let playlist2_content = "artist1/album1/title1.flac\nartist2/album2/title1.flac";
        let playlist2_path = music_dir.join("playlist2.m3u8");
        create_test_file(&playlist2_path, playlist2_content);

        let playlist1_path = music_dir.join("playlist.m3u8");

        // Verify files exist before deletion
        assert!(playlist1_path.exists());
        assert!(playlist2_path.exists());
        assert!(music_dir.join("artist1/album1/title1.flac").exists());
        assert!(music_dir.join("artist2/album2/title1.flac").exists());

        let mut cmd = Command::cargo_bin("plm-delete-playlist").unwrap();
        let assert = cmd
            .arg("--media")
            .arg(playlist1_path.to_str().unwrap())
            .arg(playlist2_path.to_str().unwrap())
            .assert();

        assert.success();

        // Verify both playlists were deleted
        assert!(!playlist1_path.exists());
        assert!(!playlist2_path.exists());

        // Verify all media files were deleted (even those referenced in both playlists)
        assert!(!music_dir.join("artist1/album1/title1.flac").exists());
        assert!(!music_dir.join("artist1/album1/title2.flac").exists());
        assert!(!music_dir.join("artist2/album1/title1.flac").exists());
        assert!(!music_dir.join("artist2/album2/title1.flac").exists());

        // Verify empty directories were deleted
        assert!(!music_dir.join("artist1/album1").exists());
        assert!(!music_dir.join("artist1").exists());
        assert!(!music_dir.join("artist2/album1").exists());
        assert!(!music_dir.join("artist2/album2").exists());
        assert!(!music_dir.join("artist2").exists());
    }

    #[test]
    fn test_delete_playlist_missing_args() {
        let mut cmd = Command::cargo_bin("plm-delete-playlist").unwrap();
        let assert = cmd.assert();

        assert.failure();
    }

    #[test]
    fn test_delete_playlist_nonexistent() {
        let temp_dir = setup_test_directory();
        let nonexistent_path = temp_dir.path().join("nonexistent.m3u8");

        let mut cmd = Command::cargo_bin("plm-delete-playlist").unwrap();
        let assert = cmd
            .arg(nonexistent_path.to_str().unwrap())
            .assert();

        assert.failure();
    }
}
