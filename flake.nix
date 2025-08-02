{
  description = "CIM Domain Git - Git repository introspection and graph extraction";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, advisory-db }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Use the current directory as source
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        commonArgs = {
          inherit src;
          strictDeps = true;

          # Build the current package
          # cargoExtraArgs = "-p cim-domain-git";

          buildInputs = with pkgs; [
            # Git2 dependencies
            openssl
            libgit2
            zlib
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
          ];
        };

        # Build *just* the cargo dependencies
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself
        cim-domain-git = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          doCheck = false; # We'll run tests separately
        });

        # Run tests
        cim-domain-git-tests = craneLib.cargoNextest (commonArgs // {
          inherit cargoArtifacts;
          partitions = 1;
          partitionType = "count";
        });
      in
      {
        checks = {
          inherit cim-domain-git cim-domain-git-tests;

          # Run clippy
          cim-domain-git-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "-p cim-domain-git --all-targets -- --deny warnings";
          });

          # Check formatting
          cim-domain-git-fmt = craneLib.cargoFmt {
            inherit src;
          };

          # Audit dependencies
          cim-domain-git-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };
        };

        packages = {
          default = cim-domain-git;
          inherit cim-domain-git;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = cim-domain-git;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks.${system};

          # Extra inputs for development
          nativeBuildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-nextest
            cargo-edit
            cargo-outdated
            cargo-audit
            cargo-license
            cargo-tarpaulin

            # Git development tools
            git
            gitui
            gh

            # General development tools
            just
            bacon
            mold
            sccache
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          RUST_BACKTRACE = 1;
          RUST_LOG = "debug";

          # Git2 environment variables
          LIBGIT2_SYS_USE_PKG_CONFIG = "1";
          PKG_CONFIG_PATH = "${pkgs.libgit2}/lib/pkgconfig:${pkgs.openssl.dev}/lib/pkgconfig";
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";

          shellHook = ''
            echo "CIM Domain Git Development Shell"
            echo "================================"
            echo ""
            echo "Available commands:"
            echo "  cargo build -p cim-domain-git    - Build the project"
            echo "  cargo test -p cim-domain-git     - Run tests"
            echo "  cargo watch    - Watch for changes and rebuild"
            echo "  cargo nextest  - Run tests with nextest"
            echo "  nix flake check - Check the flake"
            echo ""
          '';
        };
      });
}
