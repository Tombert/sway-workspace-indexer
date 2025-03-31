{
  description = "rust sway new workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
	rust = pkgs.rustc;
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "swaybg-rs";
          version = "0.1.0";
	  src = pkgs.lib.cleanSourceWith {
             src = ./.;
             filter = path: type: true;
          };

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
	    pkgs.cargo
            pkgs.pkg-config
          ];
        };
      });
}
