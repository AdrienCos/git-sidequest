{
  pkgs,
  inputs,
}:
pkgs.mkShell {
  name = "git-sidequest";
  inputsFrom = [
    (pkgs.callPackage ./default.nix {
      crane = inputs.crane;
    })
  ];
  # HACK: no-op value only used to make sure the flake inputs are not GC-ed
  # as long a profile of this flake is in the GCroots
  keepFlakeInputs = builtins.attrValues inputs;
  packages = with pkgs; [
    bats
    just
    cargo-dist
  ];
}
