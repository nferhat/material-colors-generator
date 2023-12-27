{
  description = "A rust utility to generate colors using the Material UI 3 standard";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      perSystem = {
        pkgs,
        inputs',
        ...
      }: {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [inputs'.fenix.packages.complete.toolchain inputs'.fenix.packages.rust-analyzer];
          RUST_BACKTRACE = "1";
        };
        packages.default = let
          manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
          toolchain = inputs'.fenix.packages.minimal;
        in
          (pkgs.makeRustPlatform {inherit (toolchain) cargo rustc;})
          .buildRustPackage {
            pname = manifest.name;
            version = manifest.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };
      };
    };

  nixConfig = {
    substituters = [
      "https://nix-community.cachix.org"
    ];
    trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };
}
