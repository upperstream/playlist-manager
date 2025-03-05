# Changelog

## [Unreleased][]

* Added
  * Add verbose option to `plm` and `plm-put-playlist` commands in man
    pages
  * Convert `plm-put-playlist` shell script to Rust program
  * Add integration tests for `plm-put-playlist`
  * Add `test` target to Makefile to run integration tests
  * Add `build` target to Makefile to build Rust binaries
  * Add `clean` target to Makefile to remove build artifacts
* Changed
  * Update Makefile to install Rust binary instead of shell script
  * Modify Makefile to handle installation of .exe files on Windows
  * Optimize `plm-put-playlist` to avoid copying the same media file
    multiple times when referenced in multiple playlists
  * Update integration test to reflect the optimized behaviour of
    `plm-put-playlist`
* Fixed
  * Fix typo in `plm.1` man page where it referred to "plm-put-command"
    instead of "plm-put-playlist"
  * Update date in `plm.1` man page to match `plm-put-playlist.1`
  * Fix inconsistency in `plm` shell script where `-v` option set
    verbosity to `-v1` instead of `-v` as expected by the Rust
    implementation

## [0.0.4][] - 20241102

* Added
  * Add uninstallation to the [Makefile](Makefile)

## [0.0.3][] - 2024-10-31

* Changed
  * Wording in document files

## [0.0.2][] - 2024-10-31

* Changed
  * Accept playlist file of which line ending is DOS style (ending with
    0x0d 0x0a)

## [0.0.1][] - 2024-10-29

* Added
  * New release
  * Supported feature:
    * Copy playlist files and associated media files to device
      (direct access to MTP device is not supported)

[Unreleased]:
  https://github.com/upperstream/playlist-manager/compare/0.0.4...HEAD
[0.0.4]:
  https://github.com/upperstream/playlist-manager/compare/0.0.3...0.0.4
[0.0.3]:
  https://github.com/upperstream/playlist-manager/compare/0.0.2...0.0.3
[0.0.2]:
  https://github.com/upperstream/playlist-manager/compare/0.0.1...0.0.2
[0.0.1]:
  https://github.com/upperstream/playlist-manager/releases/tag/0.0.1
