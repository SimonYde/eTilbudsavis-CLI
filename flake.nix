{
  description = "A basic flake with a shell";

  inputs = {

    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      fenix,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        rustBuildToolchain = fenix.packages.${system}.stable.minimalToolchain;
        rustDevToolchain = fenix.packages.${system}.stable.toolchain;
        rootCargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        rootPackage = rootCargoToml.workspace.package or rootCargoToml.package or null;
        longDescription =
          if rootPackage ? readme then builtins.readFile (./. + ("/" + rootPackage.readme)) else null;
        homepage = rootPackage.homePage or rootPackage.repository or null;
        license = rootPackage.license or null;

        rustPackage =
          bin-dir: features:
          with builtins;
          let
            cargoToml = fromTOML (readFile (bin-dir + "/Cargo.toml"));
            pname = cargoToml.package.name;
            inherit (cargoToml.package) version;
            description = cargoToml.package.description or null;
          in
          with pkgs;
          (makeRustPlatform {
            cargo = rustBuildToolchain;
            rustc = rustBuildToolchain;
          }).buildRustPackage
            {
              inherit pname version;
              src = lib.cleanSource ./.;
              cargoLock.lockFile = ./Cargo.lock;
              buildFeatures = features;
              buildInputs = [ openssl ];
              nativeBuildInputs = with pkgs; [
                pkg-config
                makeWrapper
              ];

              cargoBuildFlags = [
                "-p"
                cargoToml.package.name
              ];

              postFixup = ''
                wrapProgram $out/bin/etb \
                  --prefix LD_LIBRARY_PATH : ${pkgs.openssl.out}/lib
              '';

              meta = lib.attrsets.filterAttrs (k: v: v != null) {
                inherit
                  homepage
                  license
                  description
                  longDescription
                  ;
                mainProgram = "etb";
              };
            };
      in
      {
        packages = {
          etilbudsavis-cli = rustPackage ./. [ ];
          default = self.packages.${system}.etilbudsavis-cli;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];

          packages = with pkgs; [
            rust-analyzer
            rustDevToolchain
            cargo-audit
            cargo-deny
          ];
        };
      }
    );
}
