# Changelog

## [Unreleased][]

* Changed:
  * Modify `print_message()` in `plm-put-playlist.rs` to use the format
    of "({}-L/{})" for lyrics or "({}-M/{})" for media regardless of
    `copy_lyrics` value, removing the `copy_lyrics` argument from this
    function
  * Introduce `CommandOptions` struct in `plm-put-playlist.rs` to hold
    command line flags (verbose, copy_lyrics, keep_going) and use it as
    a parameter in functions instead of passing these three boolean
    values separately
  * Refactor `plm-put-playlist.rs` to move retry-related functions to a
    separate module `plm_put_playlist_retry` for better code
    organization and maintainability

## [v0.3.0][] - 2025-04-24

* Added:
  * Add 'version' subcommand to `plm` to display version information and
    quit, as an alternative to the `-V/--version` flags

## [v0.2.1][] - 2025-04-23

* Added:
  * Add new output message format for `plm-put-playlist` when both `-l`
    and `-v` options are given:
    * For playlist files: `({}/{}) Copy playlist {} to {}`
    * For media files: `({}-M/{}) Copy track {} to {}`
    * For lyrics files: `({}-L/{}) Copy lyrics {} to {}`
  * Update output message format for media files when only `-v` option
    is given: `({}/{}) Copy track {} to {}`
  * Add validation of error file creation at the beginning of
    `plm-put-playlist` command to fail fast if the error file cannot be
    created
  * Update documentation to mention that the error file will be empty
    if the operation completes successfully without errors
  * Add `-k/--keep-going` option to `plm-delete-playlist` command to
    continue operation despite errors, similar to the same option in
    `plm-put-playlist` command
* Changed:
  * Change version tag formatting to use 'v' prefix
  * Refactor `plm-put-playlist` to extract functions for better code
    organization and maintainability:
    * Extract `copy_single_media_file()` from `copy_media_files()`
    * Extract `retry_playlist()` and `retry_media_file()` from
      `retry_operations()`
    * Extract `process_normal_operations()` from the main function
  * Rename test files to clarify they are integration tests:
    * Rename `tests/common.rs` to `tests/integration_test_common.rs`
    * Rename `tests/put_playlist_tests.rs` to
      `tests/integration_put_playlist_tests.rs`
    * Rename `tests/delete_playlist_tests.rs` to
      `tests/integration_delete_playlist_tests.rs`
  * Strip executables during installation
* Fixed:
  * Fix file counting in `plm-put-playlist` to ensure the n-th file to
    copy is the n-th file of all files to copy across all playlist
    files, and only successfully copied files are counted in the
    sequence

## [v0.2.0][] - 2025-04-14

* Added
  * Add documentation for Playlist Manager commands and directory
    structure
  * Add `-k/--keep-going` option to `plm-put-playlist` command to
    continue operation despite errors and display results in the form of
    "(a/b) playlist copied" and "(c/d) media files copied"
  * Add `-e/--error-files` option to `plm-put-playlist` command to
    write the list of failed files to a specified file when used with
    `-k/--keep-going` option, with "P " prefix for failed playlists and
    "M " prefix for failed media files
  * Add `-r/--retry` option to `plm-put-playlist` command to retry
    failed operations from an error file produced by `-e/--error-files`
    option
* Changed
  * Improve build process
  * Remove error message when lyrics file is not found
  * Update integration tests to reflect removal of error message when
    lyrics file is not found
  * Refactor test files into separate modules for better organisation:
    * `tests/common.rs` - Common utility functions and test setup code
    * `tests/put_playlist_tests.rs` - Tests for the `plm-put-playlist`
      command
    * `tests/delete_playlist_tests.rs` - Tests for the
      `plm-delete-playlist` command
  * Modify `plm-put-playlist` to process copying media files for each
    playlist one-by-one while still maintaining the optimisation to
    avoid copying duplicate files
  * Update man page and documentation to add a separate synopsis for the
    retry operation (`-r/--retry` option) in `plm-put-playlist` command
  * Update documentation to reflect that the `-r/--retry` option works
    with the `-l/--lyrics` option in `plm-put-playlist` command
* Fixed
  * Fix install target to include Cargo.toml in version embedding

## [v0.1.0][] - 2025-03-20

* Added
  * Add `-l/--lyrics` option to `plm-put-playlist` command to copy
    lyrics files (with `.lrc` extension) along with media files
  * Add verbose option to `plm` and `plm-put-playlist` commands in man
    pages
  * Convert `plm-put-playlist` shell script to Rust program
  * Add integration tests for `plm-put-playlist`
  * Add `test` target to Makefile to run integration tests
  * Add `build` target to Makefile to build Rust binaries
  * Add `clean` target to Makefile to remove build artifacts
  * Add `-V/--version` option to `plm` command to display version
    information
  * Add `plm-delete-playlist` command to delete playlist files and
    associated media files from device
* Changed
  * Update Makefile to install Rust binary instead of shell script
  * Modify Makefile to handle installation of .exe files on Windows
  * Optimize `plm-put-playlist` to avoid copying the same media file
    multiple times when referenced in multiple playlists
  * Update integration test to reflect the optimized behaviour of
    `plm-put-playlist`
  * Enhance `clean` target in Makefile to also remove executables from
    the libexec/playlist-manager directory
* Fixed
  * Fix typo in `plm.1` man page where it referred to "plm-put-command"
    instead of "plm-put-playlist"
  * Update date in `plm.1` man page to match `plm-put-playlist.1`
  * Fix inconsistency in `plm` shell script where `-v` option set
    verbosity to `-v1` instead of `-v` as expected by the Rust
    implementation

## [v0.0.4][] - 2024-11-02

* Added
  * Add uninstallation to the [Makefile](Makefile)

## [v0.0.3][] - 2024-10-31

* Changed
  * Wording in document files

## [v0.0.2][] - 2024-10-31

* Changed
  * Accept playlist file of which line ending is DOS style (ending with
    0x0d 0x0a)

## [v0.0.1][] - 2024-10-29

* Added
  * New release
  * Supported feature:
    * Copy playlist files and associated media files to device
      (direct access to MTP device is not supported)

[Unreleased]:
  https://github.com/upperstream/playlist-manager/compare/v0.3.0...HEAD
[v0.3.0]:
  https://github.com/upperstream/playlist-manager/compare/v0.2.1...v0.3.0
[v0.2.1]:
  https://github.com/upperstream/playlist-manager/compare/v0.2.0...v0.2.1
[v0.2.0]:
  https://github.com/upperstream/playlist-manager/compare/v0.1.0...v0.2.0
[v0.1.0]:
  https://github.com/upperstream/playlist-manager/compare/v0.0.4...v0.1.0
[v0.0.4]:
  https://github.com/upperstream/playlist-manager/compare/v0.0.3...v0.0.4
[v0.0.3]:
  https://github.com/upperstream/playlist-manager/compare/v0.0.2...v0.0.3
[v0.0.2]:
  https://github.com/upperstream/playlist-manager/compare/v0.0.1...v0.0.2
[v0.0.1]:
  https://github.com/upperstream/playlist-manager/releases/tag/v0.0.1
