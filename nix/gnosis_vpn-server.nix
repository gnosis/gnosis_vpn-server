# gnosis_vpn-server.nix — GnosisVPN Server package definitions
#
# Package definitions using HOPR nix-lib build tools.
# Uses nixLib.mkRustPackage for consistent, reproducible builds across platforms.
#
# Structure:
# - Local builds: binary-gnosis_vpn-server (release), binary-gnosis_vpn-server-dev (dev)
# - Cross-compiled: binary-gnosis_vpn-server-{arch}-{os} for each target platform
# - Docker: docker-gnosis_vpn-server-{arch}-{os} — shell apps that run `docker build`
# - QA: gnosis_vpn-server-{test,clippy,docs,audit,licenses}
{
  lib,
  nixLib,
  self,
  pkgs,
  craneLib,
  advisory-db,
}:

let
  fs = lib.fileset;
  rev = toString (self.shortRev or self.dirtyShortRev);

  builders = nixLib.mkRustBuilders {
    rustToolchainFile = ../rust-toolchain.toml;
  };

  sources = {
    main = nixLib.mkSrc {
      inherit fs;
      root = ../.;
    };
    test = nixLib.mkTestSrc {
      inherit fs;
      root = ../.;
    };
    deps = nixLib.mkDepsSrc {
      inherit fs;
      root = ../.;
    };
    # Includes license/audit config files needed by crane-based checks
    checks = nixLib.mkSrc {
      inherit fs;
      root = ../.;
      extraFiles = [
        ../deny.toml
      ];
    };
  };

  mkBuildArgs =
    {
      src,
      depsSrc,
      extraCargoArgs ? "",
    }:
    {
      inherit src depsSrc rev;
      prependPackageName = false;
      cargoExtraArgs = "--bin gnosis_vpn-server ${extraCargoArgs}";
      cargoToml = ../Cargo.toml;
    };

  # Creates a shell application that builds the Docker image using `docker build`.
  # The binary argument MUST be a fully static musl binary (built by builders.x86_64-linux
  # or builders.aarch64-linux) — nix-lib configures those builders with:
  #   crossSystem = musl64 / aarch64-multiplatform-musl
  #   isStatic    = true
  # which sets CARGO_BUILD_TARGET=*-unknown-linux-musl and
  # CARGO_BUILD_RUSTFLAGS="-C target-feature=+crt-static".
  # The resulting binary has no shared-library dependencies and runs on the
  # Alpine base image in docker/Dockerfile without a glibc compatibility layer.
  mkDockerBuildScript =
    {
      platform,
      archTag,
      binary,
    }:
    pkgs.writeShellApplication {
      name = "docker-gnosis_vpn-server-${platform}";
      runtimeInputs = [
        pkgs.docker
        pkgs.coreutils
      ];
      text = ''
        BUILD_DIR=$(mktemp -d)
        cleanup() { rm -rf "$BUILD_DIR"; }
        trap cleanup EXIT

        cp -r ${../docker}/. "$BUILD_DIR/"
        install -m755 ${../docker/wrapper.sh} "$BUILD_DIR/wrapper.sh"
        install -m755 ${binary}/bin/gnosis_vpn-server "$BUILD_DIR/gnosis_vpn-server"

        echo "[+] Building: gnosis_vpn-server:${platform}"
        docker build --platform ${archTag} -t "gnosis_vpn-server:latest" "$BUILD_DIR/"
        echo "[✓] Done: gnosis_vpn-server:${platform}"
      '';
    };

  binary-gnosis_vpn-server = builders.local.callPackage nixLib.mkRustPackage (mkBuildArgs {
    src = sources.main;
    depsSrc = sources.deps;
  });

  binary-gnosis_vpn-server-dev = builders.local.callPackage nixLib.mkRustPackage (
    (mkBuildArgs {
      src = sources.main;
      depsSrc = sources.deps;
    })
    // {
      CARGO_PROFILE = "dev";
    }
  );

  # Cross-compiled — x86_64 Linux
  binary-gnosis_vpn-server-x86_64-linux =
    builders.x86_64-linux.callPackage nixLib.mkRustPackage
      (mkBuildArgs {
        src = sources.main;
        depsSrc = sources.deps;
      });

  binary-gnosis_vpn-server-x86_64-linux-dev = builders.x86_64-linux.callPackage nixLib.mkRustPackage (
    (mkBuildArgs {
      src = sources.main;
      depsSrc = sources.deps;
    })
    // {
      CARGO_PROFILE = "dev";
    }
  );

  # Cross-compiled — aarch64 Linux
  binary-gnosis_vpn-server-aarch64-linux =
    builders.aarch64-linux.callPackage nixLib.mkRustPackage
      (mkBuildArgs {
        src = sources.main;
        depsSrc = sources.deps;
      });

  binary-gnosis_vpn-server-aarch64-linux-dev =
    builders.aarch64-linux.callPackage nixLib.mkRustPackage
      (
        (mkBuildArgs {
          src = sources.main;
          depsSrc = sources.deps;
        })
        // {
          CARGO_PROFILE = "dev";
        }
      );

in
{
  # Local builds
  inherit binary-gnosis_vpn-server binary-gnosis_vpn-server-dev;

  # Cross-compiled — x86_64 Linux
  inherit binary-gnosis_vpn-server-x86_64-linux binary-gnosis_vpn-server-x86_64-linux-dev;

  # Cross-compiled — aarch64 Linux
  inherit binary-gnosis_vpn-server-aarch64-linux binary-gnosis_vpn-server-aarch64-linux-dev;

  # Docker build scripts — shell apps that invoke `docker build`.
  # Each uses the corresponding musl/static cross-compiled binary (see mkDockerBuildScript
  # comment above), which is Alpine-compatible without any glibc layer.
  docker-gnosis_vpn-server-x86_64-linux = mkDockerBuildScript {
    platform = "x86_64-linux";
    archTag = "linux/amd64";
    binary = binary-gnosis_vpn-server-x86_64-linux; # musl static — see builders.x86_64-linux
  };

  docker-gnosis_vpn-server-aarch64-linux = mkDockerBuildScript {
    platform = "aarch64-linux";
    archTag = "linux/arm64";
    binary = binary-gnosis_vpn-server-aarch64-linux; # musl static — see builders.aarch64-linux
  };

  # Tests / QA
  gnosis_vpn-server-test = builders.local.callPackage nixLib.mkRustPackage (
    (mkBuildArgs {
      src = sources.test;
      depsSrc = sources.deps;
    })
    // {
      runTests = true;
    }
  );

  gnosis_vpn-server-clippy = builders.local.callPackage nixLib.mkRustPackage (
    (mkBuildArgs {
      src = sources.main;
      depsSrc = sources.deps;
    })
    // {
      runClippy = true;
    }
  );

  gnosis_vpn-server-docs = builders.localNightly.callPackage nixLib.mkRustPackage (
    (mkBuildArgs {
      src = sources.main;
      depsSrc = sources.deps;
    })
    // {
      buildDocs = true;
    }
  );

  # Audit dependencies
  gnosis_vpn-server-audit = craneLib.cargoAudit {
    src = sources.checks;
    inherit advisory-db;
  };

  # Audit licenses
  gnosis_vpn-server-licenses = craneLib.cargoDeny {
    src = sources.checks;
  };
}
// lib.optionalAttrs pkgs.stdenv.isDarwin {
  # macOS — aarch64 (only available on Darwin hosts; cctools is Darwin-only)
  binary-gnosis_vpn-server-aarch64-darwin =
    builders.aarch64-darwin.callPackage nixLib.mkRustPackage
      (mkBuildArgs {
        src = sources.main;
        depsSrc = sources.deps;
      });

  binary-gnosis_vpn-server-aarch64-darwin-dev =
    builders.aarch64-darwin.callPackage nixLib.mkRustPackage
      (
        (mkBuildArgs {
          src = sources.main;
          depsSrc = sources.deps;
        })
        // {
          CARGO_PROFILE = "dev";
        }
      );
}
