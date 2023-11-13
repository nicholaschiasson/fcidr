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

```
cargo install fcidr
```

### Usage

```
Fragmented Classless Inter-Domain Routing (FCIDR)

Usage: fcidr [CIDR] <COMMAND>

Commands:
  complement  Compute the complement of the input CIDR(s) [aliases: !, not]
  difference  Compute the set difference between the input CIDR(s) and another CIDR [aliases: -, exclude, minus]
  superset    Exits successfully if the input CIDR(s) is a superset of another CIDR [aliases: >, contains]
  union       Compute the set union of the input CIDR(s) and another CIDR [aliases: +, include, plus]
  help        Print this message or the help of the given subcommand(s)

Arguments:
  [CIDR]  The input CIDR range and first operand to the computation. If omitted, input is taken from stdin. In this way, multiple computations can be chained together

Options:
  -h, --help     Print help
  -V, --version  Print version`
```

### Examples

#### Computing a specific set of CIDRs within 10.0.0.0/8, excluding some subranges, and including a subrange of one that was excluded

```
fcidr 10.0.0.0/8 difference 10.0.64.0/20 | fcidr difference 10.0.82.0/24 | fcidr union 10.0.82.74/31
10.0.0.0/18
10.0.80.0/23
10.0.82.74/31
10.0.83.0/24
10.0.84.0/22
10.0.88.0/21
10.0.96.0/19
10.0.128.0/17
10.1.0.0/16
10.2.0.0/15
10.4.0.0/14
10.8.0.0/13
10.16.0.0/12
10.32.0.0/11
10.64.0.0/10
10.128.0.0/9
```

#### Inverting the previous result

```
fcidr 10.0.0.0/8 difference 10.0.64.0/20 | fcidr difference 10.0.82.0/24 | fcidr union 10.0.82.74/31 | fcidr complement
0.0.0.0/5
8.0.0.0/7
10.0.64.0/20
10.0.82.0/26
10.0.82.64/29
10.0.82.72/31
10.0.82.76/30
10.0.82.80/28
10.0.82.96/27
10.0.82.128/25
11.0.0.0/8
12.0.0.0/6
16.0.0.0/4
32.0.0.0/3
64.0.0.0/2
128.0.0.0/1
```

#### Alternative concise syntax

Note these symbols may not play nice with your shell, so you can quote them if you want, for example `fcidr "!"`).

```
fcidr 10.0.0.0/8 + 127.0.0.0/16 | fcidr - 10.64.0.0/16 | fcidr !
0.0.0.0/5
8.0.0.0/7
10.64.0.0/16
11.0.0.0/8
12.0.0.0/6
16.0.0.0/4
32.0.0.0/3
64.0.0.0/3
96.0.0.0/4
112.0.0.0/5
120.0.0.0/6
124.0.0.0/7
126.0.0.0/8
127.1.0.0/16
127.2.0.0/15
127.4.0.0/14
127.8.0.0/13
127.16.0.0/12
127.32.0.0/11
127.64.0.0/10
127.128.0.0/9
128.0.0.0/1
```

#### Check if an IP is within a CIDR

```
fcidr 255.0.0.0/16 contains "255.0.1.2/32" && echo Woohoo!
Woohoo!
```

```
echo 255.0.0.0/16 | fcidr contains "255.1.1.2/32" && echo Woohoo!
Error: "not a superset of 255.1.1.2/32"
```

#### Check if a CIDR is within any of a large set of CIDRs

Expanding upon the previous example, thanks to Amazon publishing a JSON formatted list of their public IP ranges, we can check if an IP or CIDR effectively is owned by Amazon. As long as we can get the list of ranges separated by new lines, piping that to `fcidr` makes the task trivial.

```
curl -s https://ip-ranges.amazonaws.com/ip-ranges.json | jq -r '.prefixes[].ip_prefix' | fcidr contains 52.43.76.84/30 && echo "This CIDR is within an Amazon range."
This CIDR is within an Amazon range.
```

```
curl -s https://ip-ranges.amazonaws.com/ip-ranges.json | jq -r '.prefixes[].ip_prefix' | fcidr contains 62.43.76.0/24 && echo "This CIDR is within an Amazon range."
Error: "not a superset of 62.43.76.0/24"
```

## Development

### Prerequisites

- [nix](https://nixos.org/download.html)
- [nix flakes](https://nixos.wiki/wiki/Flakes#Enable_flakes)

### How-to

Create the development shell environment. Necessary to run all other commands.

```shell
nix develop
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
