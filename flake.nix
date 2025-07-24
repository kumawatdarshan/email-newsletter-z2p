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
  }: let
    meta = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
    inherit (meta) name version;
    overlays = [fenix.overlays.default];

    pcs = pkgs: import process-compose-flake.lib {inherit pkgs;};

    postgres-service = pkgs:
      (pcs pkgs).evalModules {
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

    ci-config-to-env = pkgs:
      pkgs.writeShellApplication {
        name = "config-to-env";
        runtimeInputs = [pkgs.yq-go];
        text = ''
          #!/usr/bin/env bash
          set -euo pipefail

          CONFIG_FILE="./configuration.yaml"

          APP_PORT=$(yq '.application_port' "$CONFIG_FILE")
          DB_HOST=$(yq '.database.host' "$CONFIG_FILE")
          DB_PORT=$(yq '.database.port' "$CONFIG_FILE")
          USER=$(yq '.database.username' "$CONFIG_FILE")
          DB_USER_PWD=$(yq '.database.password' "$CONFIG_FILE")
          DB_NAME=$(yq '.database.db_name' "$CONFIG_FILE")

          cat <<-EOF
          APP_PORT=$APP_PORT
          DB_HOST=$DB_HOST
          DB_PORT=$DB_PORT
          USER=$USER
          DB_USER_PWD=$DB_USER_PWD
          DB_NAME=$DB_NAME
          EOF
        '';
      };
  in
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      postgres = postgres-service pkgs;
      config-to-env = ci-config-to-env pkgs;
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
          config-to-env
          rustToolchain
          sqlx-cli
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
            inherit version;
            pname = name;
            inherit cargoArtifacts;
            RUSTFLAGS = "-C link-arg=-fuse-ld=mold -C target-cpu=native";
          });
      };

      devShells.default = pkgs.mkShell {
        inherit nativeBuildInputs buildInputs;
        inputsFrom = [
          postgres.config.services.outputs.devShell
        ];

        packages = with pkgs; [
          postgres.config.outputs.package
          just
          curlie
          cargo-watch
          cargo-expand
        ];
        LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath [
          pkgs.openssl
        ];
        shellHook = ''
          eval "$(config-to-env)"
          export DATABASE_URL="postgres://$USER:$DB_USER_PWD@$DB_HOST:$DB_PORT/$DB_NAME"
          export SQLX_OFFLINE=true
        '';
      };
    });
}
