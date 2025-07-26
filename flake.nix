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
            cli.options.port = 8084;
            services.postgres."pg_master" = {
              enable = true;
              superuser = "postgres";
            };
          }
        ];
      };

    config = builtins.fromJSON (builtins.readFile ./configuration/base.json);
    db = config.database;
  in
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      postgres = postgres-service pkgs;
      rustToolchain = pkgs.fenix.stable.withComponents [
        "cargo"
        "clippy"
        "rust-src"
        "rustc"
        "rustfmt"
        "rust-analyzer"
      ];

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      src = pkgs.lib.fileset.toSource {
        root = ./.;
        fileset = pkgs.lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./.)
          ./migrations
          ./configuration
          ./.sqlx
        ];
      };

      buildInputs = with pkgs; [
        openssl
      ];
      nativeBuildInputs = with pkgs;
        [
          openssl
          pkg-config
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
      packages = rec {
        docker = let
          bin = "${default}/bin/${name}";
          runtimeDirs = [
            {
              name = "configuration";
              path = ./configuration;
            }
            {
              name = "migrations";
              path = ./migrations;
            }
          ];
          runtime = pkgs.linkFarm "config" runtimeDirs;
        in
          pkgs.dockerTools.buildLayeredImage {
            inherit name;
            tag = "v${version}";
            contents = [
              runtime
            ];
            config = {
              Entrypoint = [bin];
              ExposedPorts."8000/tcp" = {};
            };
          };
        default = craneLib.buildPackage (commonArgs
          // {
            inherit version cargoArtifacts buildInputs nativeBuildInputs;
            doCheck = false;
            pname = name;
            SQLX_OFFLINE = true;
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
          export DATABASE_URL="postgres://${db.username}:${db.password}@${db.host}:${db.port}/${db.name}"
          export SQLX_OFFLINE=true
        '';
      };
    });
}
