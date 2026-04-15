{
  description = "Gnosis VPN server application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
    };
    crane = {
      url = "github:ipetkov/crane";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    # HOPR Nix Library (provides reusable Rust build functions and treefmt config)
    nix-lib = {
      url = "github:hoprnet/nix-lib";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.crane.follows = "crane";
      inputs.rust-overlay.follows = "rust-overlay";
    };
  };

  outputs =
    inputs@{
      self,
      flake-parts,
      nixpkgs,
      rust-overlay,
      crane,
      advisory-db,
      nix-lib,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.nix-lib.flakeModules.default
      ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      perSystem =
        {
          config,
          self',
          inputs',
          lib,
          system,
          ...
        }:
        let
          pkgs = import nixpkgs {
            localSystem = system;
            overlays = [ (import rust-overlay) ];
          };

          nixLib = nix-lib.lib.${system};

          craneLib = (crane.mkLib pkgs).overrideToolchain (
            p:
            (p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
              targets = [ ];
            }
          );

          gnosisVpnServerPackages = import ./nix/gnosis_vpn-server.nix {
            inherit
              lib
              nixLib
              self
              pkgs
              craneLib
              advisory-db
              ;
          };

        in
        {
          # nix-lib's flake module sets up treefmt and formatter automatically.
          # nix-lib already covers: rustfmt, nixfmt, taplo, yamlfmt, shfmt, prettier.
          nix-lib.treefmt = {
            projectRootFile = "LICENSE";
            globalExcludes = [
              "LATEST"
              "target/*"
              "modules/*"
            ];
            extraFormatters = {
              settings.formatter.prettier.excludes = [
                "*.toml"
                "*.yml"
                "*.yaml"
              ];
              programs.yamlfmt.settings = {
                formatter.type = "basic";
                formatter.max_line_length = 120;
                formatter.trim_trailing_whitespace = true;
                formatter.include_document_start = true;
                formatter.retain_line_breaks = true;
                formatter.retain_line_breaks_all = true;
              };
              programs.shellcheck.enable = true;
              programs.shfmt.indent_size = 4;
            };
          };

          checks = {
            inherit (gnosisVpnServerPackages)
              gnosis_vpn-server-clippy
              gnosis_vpn-server-docs
              gnosis_vpn-server-test
              gnosis_vpn-server-audit
              gnosis_vpn-server-licenses
              ;
          };

          packages = {
            inherit (gnosisVpnServerPackages)
              binary-gnosis_vpn-server
              binary-gnosis_vpn-server-dev
              binary-gnosis_vpn-server-x86_64-linux
              binary-gnosis_vpn-server-x86_64-linux-dev
              binary-gnosis_vpn-server-aarch64-linux
              binary-gnosis_vpn-server-aarch64-linux-dev
              docker-gnosis_vpn-server-x86_64-linux
              docker-gnosis_vpn-server-aarch64-linux
              ;
            default = gnosisVpnServerPackages.binary-gnosis_vpn-server;
          }
          // lib.optionalAttrs pkgs.stdenv.isDarwin {
            inherit (gnosisVpnServerPackages)
              binary-gnosis_vpn-server-aarch64-darwin
              binary-gnosis_vpn-server-aarch64-darwin-dev
              ;
          };

          devShells.default = craneLib.devShell {
            checks = self.checks.${system};
            packages = [ ];
          };
        };
      flake = { };
    };
}
