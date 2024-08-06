{
  pkgs,
  lib,
  stdenv,
  fenix,
}: let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
  toolchain =
    fenix.fromToolchainFile
    {
      file = ./rust-toolchain.toml;
      sha256 = "sha256-6eN/GKzjVSjEhGO9FhWObkRFaE1Jf+uqMSdQnb8lcB4=";
    };
in
  (pkgs.makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  })
  .buildRustPackage {
    pname = manifest.name;
    version = manifest.version;
    cargoLock.lockFile = ./Cargo.lock;
    doCheck = false;
    src = pkgs.lib.cleanSource ./.;
    nativeBuildInputs = [
      pkgs.pkg-config
    ];
    buildInputs =
      []
      ++ lib.optionals stdenv.isDarwin [
        pkgs.darwin.apple_sdk.frameworks.Security
      ];
  }
