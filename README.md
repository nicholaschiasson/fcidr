# fcidr

[![crates.io](https://img.shields.io/crates/v/fcidr)](https://crates.io/crates/fcidr)

Fragmented Classless Inter-Domain Routing (FCIDR)

A library exposing a data structure to represent a set of CIDR ranges as well
as an interface to compute set operations over CIDRs.

This data structure can be applied, for example, in configuring firewalls that
*implicitly deny* (AWS Security Groups) using a rule set that explicitly
expresses rules for both allow and deny.

> **Note**
> Currently, only IPv4 is supported. IPv6 support is tracked by [#6](https://github.com/nicholaschiasson/fcidr/issues/6).

## CLI

This project also publishes a binary application for use on the command line to
support composing chains of set operations on CIDRs by reading from standard
input.

### Installation

For now, crates.io is the only place this is being distributed.

The CLI tool is published as a separate crate on crates.io.

```
cargo install fcidr-cli
```

For more information, see the cli package's [README.md](cli/README.md) file.

## Development

### Prerequisites

- [mise](https://mise.jdx.dev/)

### How-to

Install tooling. Necessary to run all other commands.

```shell
mise install
```

Build with cargo.

```shell
just build
```

Check the code with cargo's built-in fast static analysis.

```shell
just check
```

Remove build files.

```shell
just clean
```

Format the code.

```shell
just format
```

Check the code with clippy for better static analysis.

```shell
just lint
```

Run the application.

```shell
just run
```

Run tests with cargo's built-in test runner.

```shell
just test
```

Watch for code changes and rebuild.

```shell
just watch
```

All `just` commands can accept additional command line arguments after a `--`.

For example: run the application with a flag to report the version.

```shell
just run -- --version
```

#### Tips and Recommendations

##### Open IDE from Development Shell

To get linking to rust binaries in your IDE, you should open the development shell from your terminal and then open your IDE
from that shell session. This will carry over the development shell's environment into your IDE.

For example if you work with VSCode.

```shell
cd path/to/this/project
nix develop
code .
```

By doing this, you can install the rust-analyzer VSCode extension and it will work properly since it will be able to point to
the correct rust binaries and libraries. You will also have access in VSCode to any packages installed by the nix flake.
