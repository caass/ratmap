{
  description = "Flake to build Go binaries from a GitHub repo and provide a devshell.";

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
      };

      devShell = pkgs.mkShell {
        buildInputs = [packages.wuffs pkgs.just];
      };
    });
}
