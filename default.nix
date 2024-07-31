{
  pkgs,
  lib,
  stdenv,
}: let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
  pkgs.rustPlatform.buildRustPackage {
    pname = manifest.name;
    version = manifest.version;
    cargoLock.lockFile = ./Cargo.lock;
    doCheck = false;
    src = pkgs.lib.cleanSource ./.;
    nativeBuildInputs = [
      pkgs.pkg-config
    ];
    buildInputs =
      [
        pkgs.openssl.dev
        pkgs.libgit2
      ]
      ++ lib.optionals stdenv.isDarwin [
        pkgs.darwin.apple_sdk.frameworks.Security
      ];
  }
