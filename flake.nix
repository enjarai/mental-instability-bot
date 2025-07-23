{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain (p:
          p.rust-bin.nightly.latest.default);
      in {
        # devShells.default = import ./shell.nix { inherit pkgs; };
        packages.default = pkgs.callPackage ./package.nix {
          inherit craneLib;
        };
      }
    ) // {
      nixosModules.default = import ./module.nix {
        packages = self.packages;
      };
    };
}
