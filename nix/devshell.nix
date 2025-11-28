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
      cargo-hakari
      cargo-expand
      bunyan-rs
    ];

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
      pkgs.openssl
    ];
    DATABASE_URL = "sqlite:./data.db";
  };
}
