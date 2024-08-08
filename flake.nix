{
  description = "git-sidequest";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
  }: let
    pkgsFor = nixpkgs.legacyPackages;
    forAllSystems = nixpkgs.lib.genAttrs ["x86_64-linux" "x86_64-darwin" "aarch64-linux"];
  in {
    packages = forAllSystems (
      system: {
        default = pkgsFor.${system}.callPackage ./default.nix {
          fenix = fenix.packages.${system};
        };
      }
    );
    devShells = forAllSystems (
      system: {
        default = pkgsFor.${system}.callPackage ./shell.nix {
          fenix = fenix.packages.${system};
          inputs = self.inputs;
        };
      }
    );
  };
}
