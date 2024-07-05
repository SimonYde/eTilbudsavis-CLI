{
  description = "A basic flake with a shell";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
    inputs.flake-utils.follows = "flake-utils";
  };

  outputs =
    {
      self,
      rust-overlay,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          system = system;
          overlays = [ rust-overlay.overlays.default ];
        };
        buildInputs = [
          pkgs.rust-bin.stable."1.77.2".default
          pkgs.openssl
        ];
      in
      {
        packages = rec {
          default = pkgs.rustPlatform.buildRustPackage {
            name = "etilbudsavis-cli";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            buildInputs = buildInputs;
            nativeBuildInputs = [ pkgs.pkg-config ];
            doCheck = true;
          };
          dockerImage = pkgs.dockerTools.buildLayeredImage {
            name = "etilbudsavis-cli";
            tag = "latest";
            contents = [ default ];
          };
        };

        devShells.default = pkgs.mkShell {
          packages =
            with pkgs;
            [
              rust-analyzer
              pre-commit
              pkgs.pkg-config
            ]
            ++ buildInputs;
        };
      }
    );
}
