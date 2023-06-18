with import <nixpkgs> {};

mkShell {
  buildInputs = with elmPackages; [
    openssl
    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer
    elm
    elm-format
    elm-language-server
  ];
}
