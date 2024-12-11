{
  description = "Dev environment for Ratmap";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      patchDir = ./patches;
    in rec {
      packages.wuffs = pkgs.buildGoModule {
        pname = "wuffs";
        version = "0.4.0-alpha.9";

        vendorHash = null;

        src = pkgs.fetchFromGitHub {
          owner = "google";
          repo = "wuffs";
          rev = "v0.4.0-alpha.9";
          hash = "sha256-XbupK4QYnPudUlO5tRWrQRncGHITzJL//Yk/E7WNxYk=";
        };

        buildInputs = [pkgs.zstd pkgs.zlib pkgs.lz4];
        nativeBuildInputs = [pkgs.pkg-config];

        patches = [(patchDir + "/0001-wuffs-root-dir.patch")];
      };

      devShell = pkgs.mkShell {
        buildInputs = [packages.wuffs pkgs.just pkgs.clang];
        shellHook = ''
          export WUFFS_INCLUDE_PATH=${packages.wuffs.src}/release/c
          export WUFFS_SRC_DIR=${packages.wuffs.src}
        '';
      };
    });
}
