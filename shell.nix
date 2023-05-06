with import <nixpkgs> {};

mkShell {
  buildInputs = with elmPackages; [
    elm
    elm-format
    elm-language-server
  ];
}
