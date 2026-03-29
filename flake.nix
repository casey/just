{
  description = "Just a command runner";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };

        # Read version from Cargo.toml
        package = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
        version = package.version;

        # Common build inputs
        buildInputs = with pkgs;
          lib.optionals stdenv.hostPlatform.isDarwin [
            libiconv
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
            export JUST_PATH_PREFIX="${pkgs.lib.makeBinPath [pkgs.coreutils pkgs.bashInteractive]}''${JUST_PATH_PREFIX:+:$JUST_PATH_PREFIX}"
          '';

          meta = {
            description = package.description;
            homepage = package.homepage;
            changelog = "${package.repository}/blob/master/CHANGELOG.md";
            license = pkgs.lib.licenses.cc0;
            mainProgram = "just";
          };
        };

        # Development shell with additional tools
        devShell = pkgs.mkShell {
          inputsFrom = [just];

          packages = with pkgs; [
            # Rust toolchain
            rustc
            cargo
            clippy
            rustfmt
            rust-analyzer

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
          '';
        };
      in {
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
