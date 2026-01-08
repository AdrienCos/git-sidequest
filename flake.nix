{
  description = "git-sidequest";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    rust-overlay,
  }: let
    pkgsFor = nixpkgs.legacyPackages;
    forAllSystems = nixpkgs.lib.genAttrs ["x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin"];
  in {
    packages = forAllSystems (
      system: {
        default = (pkgsFor.${system}.extend (import rust-overlay)).callPackage ./default.nix {
          crane = crane;
        };
      }
    );
    devShells = forAllSystems (
      system: {
        default = (pkgsFor.${system}.extend (import rust-overlay)).callPackage ./shell.nix {
          inputs = self.inputs;
        };
      }
    );
  };
}
