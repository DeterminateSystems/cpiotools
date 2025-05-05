{
  description = "cpiotools";

  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";

  outputs = inputs:
    let
      lastModifiedDate = inputs.self.lastModifiedDate or inputs.self.lastModified or "19700101";
      version = "${builtins.substring 0 8 lastModifiedDate}-${inputs.self.shortRev or "dirty"}";

      allSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      forAllSystems = f: inputs.nixpkgs.lib.genAttrs allSystems (system: f {
        inherit system;
        pkgs = import inputs.nixpkgs { inherit system; };
      });
    in
    {
      devShell = forAllSystems ({ system, pkgs, ... }: inputs.self.packages.${system}.default.overrideAttrs ({ nativeBuildInputs ? [ ], ... }: {
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
            inherit version;
            src = inputs.self;
            cargoLock.lockFile = ./Cargo.lock;
          };
        });
    };
}
