{
  description = "Email Newsletter Service, guided project from Zero 2 Production, implemented in axum instead.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    services-flake.url = "github:juspay/services-flake";
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
    services-flake,
    process-compose-flake,
    crane,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [fenix.overlays.default];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      pcs = import process-compose-flake.lib {inherit pkgs;};
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
    in rec {
      inherit pcs;
      postgres-service = pcs.evalModules {
        modules = [
          services-flake.processComposeModules.default
          {
            services.postgres."pg_master" = {
              enable = true;
              superuser = "postgres";
            };
          }
        ];
      };

      packages = {
        # not really needed tbh. i think this is just wrapper around process-compose
        postgres-service-runner = postgres-service.config.outputs.package;

        default = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            RUSTFLAGS = "-C link-arg=-fuse-ld=mold -C target-cpu=native";
          });
      };

      devShells.default = pkgs.mkShell {
        inherit nativeBuildInputs buildInputs;
        inputsFrom = [
          postgres-service.config.services.outputs.devShell
        ];

        packages = with pkgs; [
          postgres-service.config.outputs.package
          just
          curlie
          cargo-watch
          cargo-expand
          sqlx-cli
        ];
        LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath [
          pkgs.openssl
        ];
        shellHook = ''
          export DATABASE_URL=postgres://postgres:@localhost:5432/newsletter
          export SQLX_OFFLINE=true
        '';
      };
    });
}
