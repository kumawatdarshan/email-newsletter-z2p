{
  projectRootFile = "flake.nix";
  programs = {
    alejandra.enable = true;
    sql-formatter.enable = false; # causing some bugs regarding changed mtime, i will figure out later
    taplo.enable = true;
    rustfmt.enable = true;
    jsonfmt.enable = true;
    just.enable = true;
  };
}
