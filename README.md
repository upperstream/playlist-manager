# Playlist Manager

The Playlist Manager aims to be able to manipulate playlists and
associated media files between PC and audio playback device.

## Feature

* Copy playlist files and associated media files from a PC to a device.
* Copy lyrics files (with `.lrc` extension) along with media files.
* Delete playlist files and associated media files from a device.

## Prerequisites

### For running the application

* POSIX compliant shell

### For building from source and installation

* Rust and Cargo (Rust package manager)
* `awk`
* `grep`
* `install`
* `make`
* `sed`

## Install

Execute `make PREFIX=... install` using either BSD make or GNU make to
install Playlist Manager.  Assigning PREFIX value specifies the
destination of installation.  Default value is `/usr/local`.

## Uninstall

Execute `make PREFIX=... uninstall` to uninstall the Playlist Manager.
Assigning PREFIX value specifies the location where to uninstall from.
Default value is `/usr/local`.

## Limitations

* This tool does not support direct access to MTP device at this moment.
  MTP device must be mounted using external tools such as [MTPfs][] so
  that device storage can be accessible as a part of PC's filesystem.

## Feature TODO list

There are couples of features to add:

* Copy albums from a PC to a device
* Delete albums on a device
* Create a playlist file with associated media files on a PC
* Direct access to MTP device
* Capture a signal `SIGUSR1` or `SIGINFO` and displays the current
  status during the operation of  `plm put-playlist` and
  `plm delete-playlist` without `-v` option
* Add an option to display a progress bar during the operation of
  `plm put-playlist` and `plm delete-playlist` without `-v` option
* Copy media files only when the source is newer than the destination

## Licensing

Files in this project are provided under the [ISC License][].
See [LICENSE.txt](LICENSE.txt) file for details.

[ISC License]:
  http://www.isc.org/downloads/software-support-policy/isc-license
[MTPfs]: https://www.adebenham.com/mtpfs/
  "MTPfs - Dual Elephants - Chris Debenhams homepage"
