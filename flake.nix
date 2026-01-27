{
  description = "Godstack Monorepo";

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

        # Backend source
        backendSrc = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./backend;
          filter = path: type:
            (craneLib.filterCargoSources path type)
            || (builtins.match ".*config.*" path != null)
            || (builtins.match ".*migrations.*" path != null);
        };

        commonArgs = {
          src = backendSrc;
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
          # ⚡ GOD MODE: Jemalloc für bessere Speicherverwaltung (10-20% schneller, stabiler)
          cargoExtraArgs = "--features jemalloc";
          postInstall = ''
            mkdir -p $out/share/godstack
            cp -r ${./backend/config} $out/share/godstack/config
            cp -r ${./backend/migrations} $out/share/godstack/migrations
          '';
        });

        godstack-api-static = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          # ⚡ GOD MODE: Jemalloc auch für statischen Build
          cargoExtraArgs = "--features jemalloc";
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
            cargo-nextest  # ⚡ PERFORMANCE: 60% schnellere Tests
            just
            sqlx-cli
            docker-compose
            # Build dependencies
            autoconf
            automake
            libtool
            jemalloc
            # ⚡ PERFORMANCE: Schnellerer Linker (3-10x schneller als ld)
            mold
            clang
            # ⚡ PERFORMANCE: Compiler Cache (macht Rebuilds fast augenblicklich)
            sccache
            # Frontend tools
            nodejs_20
            nodePackages.pnpm  # ⚡ PERFORMANCE: Schneller als npm, hardlinkt Dependencies
            # Protobuf tools
            buf
          ];

          # ⚡ PERFORMANCE: sccache als Compiler-Wrapper
          RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
          SCCACHE_CACHE_SIZE = "10G";
          
          JEMALLOC_SYS_WITH_MALLOC_CONF = "background_thread:true";
          RUST_LOG = "debug";
          RUST_BACKTRACE = "1";
        };

        checks = {
          inherit godstack-api;
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          fmt = craneLib.cargoFmt { src = backendSrc; };
        };
      }
    );
}
