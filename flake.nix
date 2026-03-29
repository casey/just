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

        package = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;

        nativeBuildInputs = with pkgs; [
          installShellFiles
          pkg-config
        ];

        just = pkgs.rustPlatform.buildRustPackage {
          pname = "just";
          version = package.version;

          src = ./.;

          auditable = false;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs;

          doCheck = false;

          postInstall = ''
            for shell in bash fish zsh; do
              $out/bin/just --completions $shell > just.$shell
              installShellCompletion just.$shell
            done

            $out/bin/just --man > just.1
            installManPage just.1
          '';

          meta = {
            description = package.description;
            homepage = package.homepage;
            changelog = "${package.repository}/blob/master/CHANGELOG.md";
            license = pkgs.lib.licenses.cc0;
            mainProgram = "just";
          };
        };

        devShell = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
          ];

          RUSTC = "${pkgs.rustc}/bin/rustc";
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

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
