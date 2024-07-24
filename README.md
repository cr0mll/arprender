# Introduction
ARPrender is a tool for performing advanced network attacks based on the [Address Resolution Protocol (ARP)](https://cyberclopaedia.gitbook.io/cyberclopaedia/networking/protocols/address-resolution-protocol-arp).

# Usage
ARPrender has different commands for performing various attacks. Note that most of them require elevated privileges or the `CAP_NET_RAW` capability.

You can get a list of them by running `arprender -h`.
```
$./arprender -h
Arprender v0.1.0
A suite for advanced ARP-based attacks.
cr0mll (C) cr0mll@protonmail.com 

Usage: arprender <COMMAND>

Commands:
  interfaces   Lists the available network interfaces
  scan         Performs an ARP scan of the network
  resolve      Resolves a single IP address to its corresponding MAC address via ARP
  impersonate  Impersonates the target host by tricking all devices on the network to forward all traffic intended for the target to you
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

To obtain help information about a given command, run

```
arprender help <command>
```

# Installation
Currently, the only supported operating systems are Linux distributions due to the dependency of [libpnet](https://github.com/libpnet/libpnet).
## Build from Source
Whilst building from source should technically be possible on Windows, you would have to manually ensure that `libpnet`'s requirements are met.

To build from source on Linux:
1. Download and install [Rust](https://www.rust-lang.org/tools/install) with Cargo.
2. Clone the ARPrender repository to your desired location.
```
git clone https://github.com/cr0mll/arprender
```
3. Change into the cloned repository and run `cargo build`. The resulting binary is called `arprender` and should be located under `target`.

