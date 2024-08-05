{pkgs}:
pkgs.mkShell {
  inputsFrom = [(pkgs.callPackage ./default.nix {})];
  packages = with pkgs; [
    bats
    just
  ];
}
