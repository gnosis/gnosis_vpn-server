{
  description = "GnosisVPN server application";

  inputs = {
    flake-utils.url = github:numtide/flake-utils;
    flake-parts.url = github:hercules-ci/flake-parts;
    nixpkgs.url = github:NixOS/nixpkgs/release-24.11;
    rust-overlay.url = github:oxalica/rust-overlay/master;
    # using a fork with an added source filter
    crane.url = github:hoprnet/crane/tb/20240117-find-filter;
    # pin it to a version which we are compatible with
    pre-commit.url = github:cachix/pre-commit-hooks.nix;
    treefmt-nix.url = github:numtide/treefmt-nix;
    flake-root.url = github:srid/flake-root;

    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    pre-commit.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, flake-parts, flake-root, rust-overlay, crane, pre-commit, treefmt-nix, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.flake-root.flakeModule
      ];
      perSystem = { config, lib, self', inputs', system, ... }:
        let
          rev = toString (self.shortRev or self.dirtyShortRev);
          fs = lib.fileset;
          localSystem = system;
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit localSystem overlays;
            packageOverrides = pkgs: {
              openssl = pkgs.openssl.override {
                static = true;
              };
            };
          };
          rustNightly = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          craneLibNightly = (crane.mkLib pkgs).overrideToolchain rustNightly;

          depsSrc = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./vendor
              ./.cargo/config.toml
              ./Cargo.lock
              (fs.fileFilter (file: file.name == "Cargo.toml") ./.)
            ];
          };

          src = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./vendor
              ./.cargo/config.toml
              ./Cargo.lock
              ./README.md
              (fs.fileFilter (file: file.hasExt "rs") ./.)
              (fs.fileFilter (file: file.hasExt "toml") ./.)
            ];
          };

          rust-builder-local = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane localSystem;
          };

          rust-builder-local-nightly = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane localSystem;
            useRustNightly = true;
          };

          rust-builder-x86_64-linux = import ./nix/rust-builder.nix
            { inherit nixpkgs rust-overlay crane localSystem; };

          rust-builder-aarch64-linux = import
            ./nix/rust-builder.nix
            {
              inherit nixpkgs rust-overlay crane localSystem;
              crossSystem = pkgs.lib.systems.examples.aarch64-multiplatform;
              isCross = true;
            };

          rust-builder-armv7l-linux = import
            ./nix/rust-builder.nix
            {
              inherit nixpkgs rust-overlay crane localSystem;
              crossSystem = pkgs.lib.systems.examples.armv7l-hf-multiplatform;
              isCross = true;
            };

          gvpnBuildArgs = {
            inherit src depsSrc rev;
            cargoExtraArgs = "--all";
            cargoToml = ./Cargo.toml;
          };

          gvpn = rust-builder-local.callPackage
            ./nix/rust-package.nix
            gvpnBuildArgs;

          gvpn-x86_64-linux = rust-builder-x86_64-linux.callPackage
            ./nix/rust-package.nix
            gvpnBuildArgs;

          gvpn-aarch64-linux = rust-builder-aarch64-linux.callPackage
            ./nix/rust-package.nix
            gvpnBuildArgs;
          gvpn-armv7l-linux = rust-builder-armv7l-linux.callPackage
            ./nix/rust-package.nix
            gvpnBuildArgs;

          gvpn-clippy = rust-builder-local.callPackage
            ./nix/rust-package.nix
            (gvpnBuildArgs // { runClippy = true; });
          gvpn-test = rust-builder-local.callPackage
            ./nix/rust-package.nix
            (gvpnBuildArgs // { runTests = true; });
          gvpn-debug = rust-builder-local.callPackage
            ./nix/rust-package.nix
            (gvpnBuildArgs // { CARGO_PROFILE = "dev"; });

          defaultDevShell = import
            ./nix/shell.nix
            { inherit pkgs config crane; };

          run-check = flake-utils.lib.mkApp
            {
              drv = pkgs.writeShellScriptBin "run-check" ''
                set -e
                check=$1
                if [ -z "$check" ]; then
                  nix flake show --json 2>/dev/null | \
                    jq -r '.checks."${system}" | to_entries | .[].key' | \
                    xargs -I '{}' nix build ".#checks."${system}".{}"
                else
                	nix build ".#checks."${system}".$check"
                fi
              '';
            };
        in
        {
          treefmt = {
            inherit (config.flake-root) projectRootFile;

            settings.global.excludes = [
              "LICENSE"
              "nix/setup-hook-darwin.sh"
              "target/*"
              "vendor/*"
            ];

            programs.shfmt.enable = true;
            settings.formatter.shfmt.includes = [ "*.sh" ];


            programs.yamlfmt.enable = true;
            settings.formatter.yamlfmt.includes = [ ".github/workflows/*.yaml" ];
            settings.formatter.yamlfmt.settings = {
              formatter.type = "basic";
              formatter.max_line_length = 120;
              formatter.trim_trailing_whitespace = true;
              formatter.scan_folded_as_literal = true;
              formatter.include_document_start = true;
            };

            programs.prettier.enable = true;
            settings.formatter.prettier.includes = [ "*.md" "*.json" ];
            settings.formatter.prettier.excludes = [ "*.yml" "*.yaml" ];

            programs.rustfmt.enable = true;

            programs.nixpkgs-fmt.enable = true;

            programs.taplo.enable = true;
          };

          checks = {
            inherit gvpn-clippy;
          };

          apps = {
            check = run-check;
          };

          packages = {
            inherit gvpn gvpn-debug;
            inherit gvpn-test;
            inherit gvpn-aarch64-linux gvpn-armv7l-linux gvpn-x86_64-linux;

            default = gvpn;
          };

          devShells.default = defaultDevShell;

          formatter = config.treefmt.build.wrapper;
        };
      # platforms which are supported as build environments
      systems = [ "x86_64-linux" "aarch64-linux" ];
    };
}
