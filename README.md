# krunvm

```krunvm``` is a CLI-based utility for creating microVMs from OCI images, using [libkrun](https://github.com/containers/libkrun) and [buildah](https://github.com/containers/buildah).

## Features

* Minimal footprint
* Fast boot time
* Zero disk image maintenance
* Zero network configuration
* Support for mapping host volumes into the guest
* Support for exposing guest ports to the host

## Demo

[![asciicast](https://asciinema.org/a/CGtTS93VsdzWwUfkY1kqVnaik.svg)](https://asciinema.org/a/CGtTS93VsdzWwUfkY1kqVnaik)

## Supported platforms

- Linux/KVM on x86_64.
- Linux/KVM on AArch64.
- macOS/Hypervisor.framework on ARM64.

## Installation

### macOS

```
brew tap slp/krun
brew install krunvm
```

### Fedora

```
dnf copr enable -y slp/libkrunfw
dnf copr enable -y slp/libkrun
dnf copr enable -y slp/krunvm
dnf install -y krunvm
```

### Building from sources

#### Dependencies

* Rust Toolchain
* [libkrun](https://github.com/containers/libkrun)
* [buildah](https://github.com/containers/buildah)
* [asciidoctor](https://github.com/asciidoctor/asciidoctor)

#### Building

```
cargo build --release
```
