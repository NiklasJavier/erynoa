{
  description = "God-Stack Backend";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
          targets = [ "x86_64-unknown-linux-musl" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = path: type:
            (craneLib.filterCargoSources path type)
            || (builtins.match ".*config.*" path != null)
            || (builtins.match ".*migrations.*" path != null);
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.libiconv
          ];
          SQLX_OFFLINE = "true";
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        godstack-api = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          postInstall = ''
            mkdir -p $out/share/godstack
            cp -r ${./config} $out/share/godstack/config
            cp -r ${./migrations} $out/share/godstack/migrations
          '';
        });

        godstack-api-static = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        });

        dockerImage = pkgs.dockerTools.buildLayeredImage {
          name = "godstack-api";
          tag = "latest";
          contents = [ godstack-api-static pkgs.cacert ];
          config = {
            Cmd = [ "/bin/godstack-api" ];
            Env = [ "RUST_LOG=info" ];
            ExposedPorts."3000/tcp" = {};
          };
        };

      in {
        packages = {
          default = godstack-api;
          static = godstack-api-static;
          docker = dockerImage;
        };

        devShells.default = craneLib.devShell {
          packages = with pkgs; [
            rustToolchain
            cargo-watch
            cargo-edit
            just
            sqlx-cli
            docker-compose
          ];
          
          RUST_LOG = "debug";
          RUST_BACKTRACE = "1";
        };

        checks = {
          inherit godstack-api;
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          fmt = craneLib.cargoFmt { inherit src; };
        };
      }
    );
}
