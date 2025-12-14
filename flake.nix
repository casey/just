{
  description = "Just a command runner";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Read version from Cargo.toml
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.package.version;

        # Common build inputs
        buildInputs = with pkgs; [
          libiconv
        ] ++ lib.optionals stdenv.hostPlatform.isDarwin [
          darwin.apple_sdk.frameworks.Security
        ];

        nativeBuildInputs = with pkgs; [
          installShellFiles
          pkg-config
        ];

        # The main just package
        just = pkgs.rustPlatform.buildRustPackage {
          pname = "just";
          inherit version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs buildInputs;

          # Don't check during build since we run tests separately
          doCheck = false;

          # Generate shell completions and man pages
          postInstall = ''
            # Generate and install shell completions
            for shell in bash fish zsh; do
              $out/bin/just --completions $shell > just.$shell
              installShellCompletion just.$shell
            done

            # Generate and install man page
            $out/bin/just --man > just.1
            installManPage just.1
          '';

          # Setup hook for runtime dependencies
          setupHook = pkgs.writeText "setup-hook.sh" ''
            export JUST_PATH_PREFIX="${pkgs.lib.makeBinPath [ pkgs.coreutils pkgs.bashInteractive ]}''${JUST_PATH_PREFIX:+:$JUST_PATH_PREFIX}"
          '';

          meta = with pkgs.lib; {
            description = "Just a command runner";
            homepage = "https://github.com/casey/just";
            changelog = "https://github.com/casey/just/blob/master/CHANGELOG.md";
            license = licenses.cc0;
            mainProgram = "just";
            maintainers = with maintainers; [ ];
          };
        };

        # Development shell with additional tools
        devShell = pkgs.mkShell {
          inputsFrom = [ just ];

          packages = with pkgs; [
            # Rust toolchain with clippy and rustfmt
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
            })
            rust-bin.nightly.latest.default

            # Development tools
            cargo-watch
            cargo-fuzz
            cargo-outdated
            cargo-udeps
            mdbook
            mdbook-linkcheck
            shellcheck

            # Runtime dependencies
            bashInteractive
            coreutils

            # Additional utilities from justfile
            python3
            nodejs
            perl
            ruby
          ];

          shellHook = ''
            echo "Just development environment"
            echo "Version: ${version}"
            echo ""
            echo "Available commands:"
            echo "  cargo build       - Build the project"
            echo "  cargo test        - Run tests"
            echo "  cargo clippy      - Run clippy lints"
            echo "  just              - Run using the local justfile"
            echo ""
          '';

          RUST_SRC_PATH = "${pkgs.rust-bin.stable.latest.rust-src}/lib/rustlib/src/rust/library";
        };

      in
      {
        packages = {
          default = just;
          just = just;
        };

        apps = {
          default = flake-utils.lib.mkApp {
            drv = just;
            exePath = "/bin/just";
          };
        };

        devShells.default = devShell;

        # Formatter for `nix fmt`
        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
