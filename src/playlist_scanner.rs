use anyhow::Result;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::iter::{Filter, FilterMap, Map};

// Internal to this crate
pub(crate) type PlaylistScanner = Map<
    Filter<
        Map<
            FilterMap<Lines<BufReader<File>>, fn(Result<String, io::Error>) -> Option<String>>,
            fn(String) -> String,
        >,
        fn(&String) -> bool,
    >,
    fn(String) -> String,
>;

// Helper functions to replace closures with function pointers
// Keep these helpers private to the module
fn process_line(line: String) -> String {
    // Remove BOM if present
    let line = if line.starts_with('\u{feff}') {
        line[3..].to_string()
    } else {
        line
    };

    // Remove carriage return if present
    if line.ends_with('\r') {
        line[..line.len() - 1].to_string()
    } else {
        line
    }
}

fn filter_line(line: &String) -> bool {
    // Skip comments and empty lines
    !(line.starts_with('#') || line.is_empty())
}

fn replace_backslash(line: String) -> String {
    // Replace backslashes with forward slashes
    line.replace('\\', "/")
}

// Only read_playlist should be public to external crates
pub fn read_playlist(file: File) -> PlaylistScanner {
    BufReader::new(file)
        .lines()
        .filter_map(Result::ok as fn(Result<String, io::Error>) -> Option<String>)
        .map(process_line as fn(String) -> String)
        .filter(filter_line as fn(&String) -> bool)
        .map(replace_backslash as fn(String) -> String)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_line_removes_bom() {
        let input = "\u{feff}test line";
        let result = process_line(input.to_string());
        assert_eq!(result, "test line");
    }

    #[test]
    fn test_process_line_removes_carriage_return() {
        let input = "test line\r";
        let result = process_line(input.to_string());
        assert_eq!(result, "test line");
    }

    #[test]
    fn test_process_line_handles_normal_text() {
        let input = "test line";
        let result = process_line(input.to_string());
        assert_eq!(result, "test line");
    }

    #[test]
    fn test_filter_line_skips_comments() {
        let input = "#This is a comment".to_string();
        assert!(!filter_line(&input));
    }

    #[test]
    fn test_filter_line_skips_empty_lines() {
        let input = "".to_string();
        assert!(!filter_line(&input));
    }

    #[test]
    fn test_filter_line_keeps_content_lines() {
        let input = "artist/album/track.flac".to_string();
        assert!(filter_line(&input));
    }

    #[test]
    fn test_replace_backslash() {
        let input = "artist\\album\\track.flac";
        let result = replace_backslash(input.to_string());
        assert_eq!(result, "artist/album/track.flac");
    }

    #[test]
    fn test_read_playlist_integration() {
        // Create a temporary file with playlist content
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "#This is a comment\n\
                           artist1\\album1\\track1.flac\r\n\
                           \n\
                           artist2/album2/track2.flac\n\
                           #Another comment\n\
                           \u{feff}artist3\\album3\\track3.flac"
        )
        .unwrap();

        // Rewind the file to the beginning
        temp_file.as_file().sync_all().unwrap();

        // Open the file for reading
        let file = File::open(temp_file.path()).unwrap();

        // Read the playlist
        let playlist_items: Vec<String> = read_playlist(file).collect();

        // Check the results - should have 3 tracks with proper formatting
        assert_eq!(playlist_items.len(), 3);
        assert_eq!(playlist_items[0], "artist1/album1/track1.flac");
        assert_eq!(playlist_items[1], "artist2/album2/track2.flac");
        assert_eq!(playlist_items[2], "artist3/album3/track3.flac");
    }
}
