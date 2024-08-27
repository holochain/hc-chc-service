{
  inputs = {
    holonix = {
      url = "github:holochain/holonix?ref=main";
      inputs.crane.follows = "crane";
      inputs.rust-overlay.follows = "rust-overlay";
    };

    nixpkgs.follows = "holonix/nixpkgs";

    # lib to build a nix package from a rust crate
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "holonix/nixpkgs";
    };

    # Rust toolchain
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "holonix/nixpkgs";
    };
  };

  outputs = inputs@{ nixpkgs, holonix, crane, rust-overlay, ... }:
    holonix.inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      # provide a dev shell for all systems that the holonix flake supports
      systems = builtins.attrNames holonix.devShells;

      perSystem = { inputs', config, system, pkgs, lib, ... }:
        {
          formatter = pkgs.nixpkgs-fmt;

          devShells.default = pkgs.mkShell {
            packages = [
              # add packages from Holonix
              inputs'.holonix.packages.holochain
              inputs'.holonix.packages.lair-keystore
              inputs'.holonix.packages.rust
              (lib.optional pkgs.stdenv.isDarwin [
                pkgs.libiconv
                pkgs.darwin.apple_sdk.frameworks.CoreFoundation
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
                pkgs.darwin.apple_sdk.frameworks.Security
              ])
            ];

            shellHook = ''
              export PS1='\[\033[1;34m\][holonix:\w]\$\[\033[0m\] '
            '';
          };

          packages.hc-chc-service =
            let
              pkgs = import nixpkgs {
                inherit system;
                overlays = [ (import rust-overlay) ];
              };

              rustToolchain = pkgs.rust-bin.stable."1.78.0".minimal;

              craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

              crateInfo = craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; };
            in
            craneLib.buildPackage {
              pname = "hc-chc-service";
              version = crateInfo.version;
              src = craneLib.cleanCargoSource (craneLib.path ./.);
              doCheck = false;

              buildInputs = [ pkgs.openssl pkgs.go ]
                ++ (lib.optionals pkgs.stdenv.isDarwin
                (with pkgs.darwin.apple_sdk.frameworks; [
                  CoreFoundation
                  SystemConfiguration
                  Security
                ]));

              nativeBuildInputs = [ pkgs.perl ];
              env = {
                OPENSSL_DIR = "${pkgs.openssl.dev}";
                OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
                OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
                CARGO_TERM_VERBOSE = "true";
                RUST_BACKTRACE = 1;
                GOROOT = "${pkgs.go}/share/go";
                PATH = lib.makeBinPath [ pkgs.go ];
              };
            };
        };
    };
}
