# Playlist Manager

The Playlist Manager aims to be able to manipulate playlists and
correspondent media files between PC and audio playback device.

## Feature

* Copy playlist files and correspondent media files from a PC to a
  device

## Prerequisites

* POSIX compliant shell
* `cp`
* `sed`

## Install

Execute `make PREFIX=... install` using either BSD make or GNU make to
install Playlist Manager.  Assigning PREFIX value to specify the
destination of installation.  Default value is `/usr/local`.

In order to perform installation, `make` and `install` are required.

### Limitations

* This tool does not support direct access to MTP device at this moment.
  MTP device must be mounted using external tools such as [MTPfs][] so
  that device storage can be accessible as a part of PC's filesystem.

## Feature TODO list

There are couples of features to add:

* Copy albums from a PC to a device
* Delete playlist files and correspondent media files on a device
* Delete albums on a device
* Create a playlist file with correspondent media files on a PC
* Direct access to MTP device

## Licensing

Files in this project are provided under the [ISC License][].
See [LICENSE.txt](LICENSE.txt) file for details.

[ISC License]:
  http://www.isc.org/downloads/software-support-policy/isc-license
[MTPfs]: https://www.adebenham.com/mtpfs/
  "MTPfs - Dual Elephants - Chris Debenhams homepage"
