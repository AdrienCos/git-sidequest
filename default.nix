{
  pkgs,
  lib,
  stdenv,
  crane,
}: let
  # Parse the toolchain.toml file to build a toolchain with the correct Rust
  # tools versions
  toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  # Setup crane with the toolchain built above
  craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
  # Arguments used when building both the dependencies and the final binary
  commonArgs = {
    src = craneLib.cleanCargoSource ./.;
    strictDeps = true;
    buildInputs = [];
    nativeBuildInputs = [
      pkgs.pkg-config
    ];
  };
  # Arguments specific to the final build, can be changed without causing a full
  # rebuild of all the dependencies.
  buildArgs = {};
in
  craneLib.buildPackage (
    commonArgs
    // buildArgs
    // {
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    }
  )
