# krunvm

```krunvm``` is a CLI-based utility for managing lightweight VMs created from OCI images, using [libkrun](https://github.com/containers/libkrun) and [buildah](https://github.com/containers/buildah).

## Features

* Minimal footprint
* Fast boot time
* Zero disk image maintenance
* Zero network configuration
* Support for mapping host volumes into the guest
* Support for exposing guest ports to the host

## Demo

[![asciicast](https://asciinema.org/a/CGtTS93VsdzWwUfkY1kqVnaik.svg)](https://asciinema.org/a/CGtTS93VsdzWwUfkY1kqVnaik)

## Installation

### macOS

```
brew tap slp/krun
brew install krunvm
```

### Fedora

```
dnf copr enable slp/libkrunfw
dnf copr enable slp/libkrun
dnf copr enable slp/krunvm
dnf install krunvm
```

### Building from sources

#### Dependencies

* Rust Toolchain
* [libkrun](https://github.com/containers/libkrun)
* [buildah](https://github.com/containers/buildah)

#### Building

```
cargo build --release
```

## Limitations

### Networking

#### Networking support is limited to TCP IPv4

The current implementation of TSI (Transparent Socket Impersonation)
in libkrun is limited to TCP and IPv4. This is expected to improve
soon.

#### Domain name resolution is broken on musl-based distributions

As a consequence of the previous point, libkrun-based VMs need to use
TCP for connecting to the DNS servers. **musl libc** does not support
domain name resolution using TCP, so on distributions based on this
library (such as Alpine), name resolution is broken.

