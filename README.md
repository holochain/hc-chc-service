# chc-service

A local web server that implements the chc (Chain Head Coordinator) interface in Rust

## Usage

```sh
nix run #.hc-chc-service --port <port> --interface <interface>
```

Cli options:
```sh
Run a local chc server

Usage: hc-chc-service [OPTIONS]

Options:
  -i, --interface <INTERFACE>  The network interface to use (e.g., 127.0.0.1). Will default to 127.0.0.1 if not passed
  -p, --port <PORT>            The port to bind to. Will default to an available port if not passed
  -h, --help                   Print help
```