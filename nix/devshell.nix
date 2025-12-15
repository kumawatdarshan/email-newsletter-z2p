{
  pkgs,
  buildInputs,
  nativeBuildInputs,
}: {
  default = pkgs.mkShell {
    inherit buildInputs nativeBuildInputs;

    packages = with pkgs; [
      just
      curlie
      cargo-watch
      cargo-nextest
      cargo-hakari
      cargo-expand
      bunyan-rs
    ];

    DATABASE_URL = "sqlite:./data.db";
  };
}
