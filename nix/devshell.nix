{
  config,
  pkgs,
  buildInputs,
  nativeBuildInputs,
  process-compose-flake,
  services-flake,
}: let
  db = config.database;
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

  postgres = postgres-service pkgs;
in {
  default = pkgs.mkShell {
    inherit buildInputs nativeBuildInputs;
    inputsFrom = [
      postgres.config.services.outputs.devShell
    ];

    packages = with pkgs; [
      postgres.config.outputs.package
      just
      curlie
      cargo-watch
      cargo-expand
      bunyan-rs
    ];

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
      pkgs.openssl
    ];

    DATABASE_URL = "postgres://${db.username}:${db.password}@${db.host}:${db.port}/${db.name}";
  };
}
