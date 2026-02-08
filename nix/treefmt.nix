{
  projectRootFile = "flake.nix";
  programs = {
    alejandra.enable = true;
    taplo.enable = true;
    rustfmt.enable = true;
    jsonfmt.enable = true;
    just.enable = true;
  };
}
