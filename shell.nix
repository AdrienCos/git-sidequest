{
  pkgs,
  fenix,
}:
pkgs.mkShell {
  name = "git-sidequest";
  inputsFrom = [(pkgs.callPackage ./default.nix {fenix = fenix;})];
  packages = with pkgs; [
    bats
    just
    cargo-dist
  ];
}
