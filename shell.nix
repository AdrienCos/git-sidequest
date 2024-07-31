{pkgs}:
pkgs.mkShell {
  inputsFrom = [(pkgs.callPackage ./default.nix {})];
  packages = with pkgs; [
    just
  ];
}
