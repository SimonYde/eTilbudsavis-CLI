{
  description = "A basic flake with a shell";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            gcc
            pkg-config
            openssl
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
            elmPackages.elm
            elmPackages.elm-format
            elmPackages.elm-language-server
          ];
        };
      });
}
