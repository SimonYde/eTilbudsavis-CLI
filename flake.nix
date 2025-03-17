{
  description = "A basic flake with a shell";

  inputs = {

    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pre-commit-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      rust-overlay,
      nixpkgs,
      flake-utils,
      pre-commit-hooks,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          system = system;
          overlays = [ rust-overlay.overlays.default ];
        };
        buildInputs = [
          pkgs.rust-bin.stable.latest.default
          pkgs.openssl
        ];
      in
      {
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              rustfmt.enable = true;
            };
          };
        };

        devShells.default = pkgs.mkShell {
          inherit (self.checks.${system}.pre-commit-check) shellHook;
          buildInputs = self.checks.${system}.pre-commit-check.enabledPackages;

          packages = buildInputs ++ [ pkgs.pkg-config ];
        };

        packages = {
          etilbudsavis-cli = pkgs.rustPlatform.buildRustPackage {
            name = "etilbudsavis-cli";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            buildInputs = buildInputs;
            nativeBuildInputs = [ pkgs.pkg-config ];
            doCheck = true;
          };

          default = self.packages.${system}.etilbudsavis-cli;
        };
      }
    );
}
