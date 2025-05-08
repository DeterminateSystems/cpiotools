{
  description = "cpiotools";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";

    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane.url = "https://flakehub.com/f/ipetkov/crane/0";
  };

  outputs = inputs:
    let
      lastModifiedDate = inputs.self.lastModifiedDate or inputs.self.lastModified or "19700101";
      version = "${builtins.substring 0 8 lastModifiedDate}-${inputs.self.shortRev or "dirty"}";

      allSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      forAllSystems = f: inputs.nixpkgs.lib.genAttrs allSystems (system: f {
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ inputs.self.overlays.default ];
        };
      });
    in
    {
      devShells = forAllSystems ({ pkgs }: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            binwalk
            cpio
            entr
            file
            nixpkgs-fmt
          ];
        };
      });

      packages = forAllSystems ({ pkgs }: {
        default = pkgs.craneLib.buildPackage {
          pname = "cpiotools";
          inherit version;
          src = inputs.self;
        };
      });

      overlays.default = final: prev:
        let
          system = prev.stdenv.hostPlatform.system;
        in
        {
          rustToolchain = with inputs.fenix.packages.${system};
            combine ([
              stable.clippy
              stable.rustc
              stable.cargo
              stable.rustfmt
              stable.rust-src
            ]);

          craneLib = inputs.crane.mkLib prev;
        };
    };
}
