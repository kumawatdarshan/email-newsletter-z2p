{
  description = "Email Newsletter Service, guided project from Zero 2 Production, implemented in axum instead.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    flake-utils,
    crane,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [fenix.overlays.default];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      rustToolchain = pkgs.fenix.stable.withComponents [
        "cargo"
        "clippy"
        "rust-src"
        "rustc"
        "rustfmt"
        "rust-analyzer"
      ];

      src = craneLib.cleanCargoSource ./.;
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
      buildInputs = with pkgs; [
        openssl
      ];

      nativeBuildInputs = with pkgs;
        [
          pkg-config
          rustToolchain
        ]
        ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
          mold
        ];
      commonArgs = {
        inherit src buildInputs nativeBuildInputs;
        strictDeps = true;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in {
      packages = {
        default = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            RUSTFLAGS = "-C link-arg=-fuse-ld=mold -C target-cpu=native";
          });
      };

      devShells.default = pkgs.mkShell {
        inherit nativeBuildInputs buildInputs;
        packages = with pkgs; [
          just
          curlie
          cargo-watch
          cargo-expand
        ];
        LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath [
          pkgs.openssl
        ];
        shellHook = ''
          export RUST_BACKTRACE="1"
          echo "use cargo build for local dev.
          nix build for distribution.
          "
        '';
      };
    });
}
