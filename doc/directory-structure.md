# Playlist Manager Directory Structure

This document provides an overview of the Playlist Manager project's
directory structure.  It explains the purpose of each directory and key
files to help developers understand the project organisation.

## Directory Hierarchy

The Playlist Manager project has the following directory structure:

```
playlist-manager/
├── bin/
│   └── plm
├── doc/
│   ├── directory-structure.md
│   ├── overview.md
│   ├── plm.md
│   ├── plm-put-playlist.md
│   └── plm-delete-playlist.md
├── libexec/
│   └── playlist-manager/
├── man/
│   └── man1/
│       ├── plm.1
│       ├── plm-put-playlist.1
│       └── plm-delete-playlist.1
├── src/
│   └── bin/
│       ├── plm-put-playlist.rs
│       └── plm-delete-playlist.rs
├── tests/
│   ├── common.rs
│   ├── put_playlist_tests.rs
│   └── delete_playlist_tests.rs
└── work/
    └── .keepme
```

## Directory Descriptions

### bin/

The `bin/` directory contains the main executable script:

- `plm` - The main command-line interface script that dispatches to
  appropriate subcommands.

### doc/

The `doc/` directory contains documentation files in Markdown format:

- `overview.md` - General overview of the Playlist Manager
- `plm.md` - Documentation for the main command
- `plm-put-playlist.md` - Documentation for the put-playlist command
- `plm-delete-playlist.md` - Documentation for the delete-playlist
  command
- `directory-structure.md` - This document, describing the project
  structure

### libexec/

The `libexec/` directory contains executable files for subcommands.  
These are typically installed in a system directory like
`/usr/local/libexec/playlist-manager/` when the project is installed.

### man/

The `man/` directory contains manual pages in the traditional Unix man
page format:

- `man1/plm.1` - Manual page for the main command
- `man1/plm-put-playlist.1` - Manual page for the put-playlist command
- `man1/plm-delete-playlist.1` - Manual page for the delete-playlist
  command

### src/

The `src/` directory contains the source code for the project:

- `bin/plm-put-playlist.rs` - Implementation of the put-playlist
  command
- `bin/plm-delete-playlist.rs` - Implementation of the delete-playlist
  command

The Playlist Manager is implemented in Rust, with each subcommand as a
separate executable.

### tests/

The `tests/` directory contains integration tests for the project:

- `common.rs` - Common utility functions and test setup code
- `put_playlist_tests.rs` - Tests for the put-playlist command
- `delete_playlist_tests.rs` - Tests for the delete-playlist command

### work/

The `work/` directory is used for temporary work files.  It contains a
`.keepme` file to ensure the directory is included in the repository.

## Key Files

### Build and Configuration Files

- `Cargo.toml` - Rust package configuration file that defines
  dependencies and build settings
- `Cargo.lock` - Lock file that ensures reproducible builds by fixing
  dependency versions
- `Makefile` - Contains targets for building, testing, installing, and
  uninstalling the project
- `.clinerules` - Defines coding and documentation standards for the
  project
- `.editorconfig` - Defines editor settings for consistent code
  formatting
- `.gitignore` - Specifies files that should be ignored by Git
- `.gitattributes` - Defines attributes for paths in the Git repository
- `embed_version.awk` - AWK script used to embed version information

### Documentation Files

- `README.md` - Project overview, installation instructions, and basic
  usage
- `CHANGELOG.md` - History of changes made to the project
- `LICENSE.txt` - Project license information (ISC License)

## Build and Execution Flow

1. Source code in `src/` is compiled using Cargo (Rust's package manager)
2. Compiled executables are placed in the `target/` directory during
   development
3. When installed, the main script from `bin/` is copied to a system
   directory like `/usr/local/bin/`
4. Executable files are installed to a system directory like
   `/usr/local/libexec/playlist-manager/`
5. Manual pages are installed to a system directory like
   `/usr/local/share/man/man1/`

The main `plm` script acts as a dispatcher, forwarding commands to the
appropriate executable in the `libexec/playlist-manager/` directory.

## Relationship to Other Documentation

For more information about the Playlist Manager, refer to:

- [overview.md](overview.md) - General overview of the Playlist Manager
- [plm.md](plm.md) - Main command documentation
- [plm-put-playlist.md](plm-put-playlist.md) - Put playlist command
  documentation
- [plm-delete-playlist.md](plm-delete-playlist.md) - Delete playlist
  command documentation
