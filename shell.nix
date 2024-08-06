{
  pkgs,
  fenix,
}:
pkgs.mkShell {
  inputsFrom = [(pkgs.callPackage ./default.nix {fenix = fenix;})];
  packages = with pkgs; [
    bats
    just
  ];
}
