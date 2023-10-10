{
  description = "cpiotools";

  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.533189.tar.gz";

  outputs =
    { self
    , nixpkgs
    , ...
    } @ inputs:
    let
      nameValuePair = name: value: { inherit name value; };
      genAttrs = names: f: builtins.listToAttrs (map (n: nameValuePair n (f n)) names);
      allSystems = [ "x86_64-linux" "aarch64-linux" "i686-linux" "x86_64-darwin" ];

      forAllSystems = f: genAttrs allSystems (system: f {
        inherit system;
        pkgs = import nixpkgs { inherit system; };
      });
    in
    {
      devShell = forAllSystems ({ system, pkgs, ... }: self.packages.${system}.package.overrideAttrs ({ nativeBuildInputs ? [ ], ... }: {
        nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
          binwalk
          entr
          file
          nixpkgs-fmt
          rustfmt
          clippy
          vim # xxd
          cpio
        ]);
      }));

      packages = forAllSystems
        ({ system, pkgs, ... }: {
          package = pkgs.rustPlatform.buildRustPackage rec {
            pname = "cpiotools";
            version = "unreleased";

            nativeBuildInputs = with pkgs; [ ];

            src = self;

            cargoLock.lockFile = src + "/Cargo.lock";
          };
        });

      defaultPackage = forAllSystems ({ system, ... }: self.packages.${system}.package);
    };
}
