{
  description = "git-sidequest";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    pkgsFor = nixpkgs.legacyPackages;
    forAllSystems = nixpkgs.lib.genAttrs ["x86_64-linux" "x86_64-darwin"];
  in {
    packages = forAllSystems (
      system: {
        default = pkgsFor.${system}.callPackage ./default.nix {};
      }
    );
    devShells = forAllSystems (
      system: {
        default = pkgsFor.${system}.callPackage ./shell.nix {};
      }
    );
  };
}
