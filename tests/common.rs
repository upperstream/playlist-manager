use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use tempfile::TempDir;

#[cfg(test)]
pub mod test_utils {
    use super::*;

    // Helper function to create a test directory structure
    pub fn setup_test_directory() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let music_dir = temp_dir.path().join("MUSIC");

        // Create directory structure
        fs::create_dir_all(music_dir.join("artist1/album1")).unwrap();
        fs::create_dir_all(music_dir.join("artist2/album1")).unwrap();
        fs::create_dir_all(music_dir.join("artist2/album2")).unwrap();

        // Create media files
        create_test_file(
            &music_dir.join("artist1/album1/title1.flac"),
            "test content 1",
        );
        create_test_file(
            &music_dir.join("artist1/album1/title2.flac"),
            "test content 2",
        );
        create_test_file(
            &music_dir.join("artist2/album1/title1.flac"),
            "test content 3",
        );
        create_test_file(
            &music_dir.join("artist2/album2/title1.flac"),
            "test content 4",
        );

        // Create lyrics files for some media files
        create_test_file(
            &music_dir.join("artist1/album1/title1.lrc"),
            "[00:00.00] Lyrics for title1",
        );
        create_test_file(
            &music_dir.join("artist2/album2/title1.lrc"),
            "[00:00.00] Lyrics for another title1",
        );

        // Create playlist file
        let playlist_content = "artist1/album1/title1.flac\nartist1/album1/title2.flac\nartist2/album1/title1.flac\nartist2/album2/title1.flac";
        create_test_file(&music_dir.join("playlist.m3u8"), playlist_content);

        temp_dir
    }

    // Helper function to create a test file with content
    pub fn create_test_file(path: &Path, content: &str) {
        let mut file = File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }
}

// Re-export the test utilities for easier imports
#[cfg(test)]
pub use test_utils::*;
