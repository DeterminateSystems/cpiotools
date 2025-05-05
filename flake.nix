{
  description = "cpiotools";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs =
    { self
    , nixpkgs
    } @ inputs:
    let
      nameValuePair = name: value: { inherit name value; };
      allSystems = [ "x86_64-linux" "aarch64-linux" "i686-linux" "x86_64-darwin" "aarch64-darwin" ];

      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
        inherit system;
        pkgs = import nixpkgs { inherit system; };
      });
    in
    {
      devShell = forAllSystems ({ system, pkgs, ... }: self.packages.${system}.default.overrideAttrs ({ nativeBuildInputs ? [ ], ... }: {
        nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
          binwalk
          entr
          file
          nixpkgs-fmt
          rustfmt
          clippy
          cpio
        ]);
      }));

      packages = forAllSystems
        ({ system, pkgs, ... }: {
          default = pkgs.rustPlatform.buildRustPackage rec {
            pname = "cpiotools";
            version = "unreleased";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
          };
        });
    };
}
