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
      patch = patchName: ./patches + "/${patchName}";
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

        patches = [(patch "0001-wuffs-root-dir.patch")];
      };

      packages.makeHeaders = pkgs.stdenv.mkDerivation {
        pname = "makeheaders";
        version = "2.25";

        src = pkgs.fetchfossil {
          url = "https://fossil-scm.org/home";
          rev = "version-2.25";
          hash = "sha256-RL5U2IsU6uexInEjlP+9qF7EzuQkMfe8e1ehCjCG+Is=";
        };

        # ./configure will attempt to build the whole fossil project, we just want `makeheaders`
        dontConfigure = true;
        nativeBuildInputs = [pkgs.gcc];

        buildPhase = ''
          gcc ./tools/makeheaders.c -o makeheaders
        '';

        installPhase = ''
          mkdir -p $out/bin
          cp makeheaders $out/bin
        '';
      };

      devShell = pkgs.mkShell {
        buildInputs = [packages.wuffs pkgs.just pkgs.clang packages.makeHeaders];
        shellHook = ''
          export WUFFS_INCLUDE_PATH=${packages.wuffs.src}/release/c
          export WUFFS_SRC_DIR=${packages.wuffs.src}
        '';
      };
    });
}
