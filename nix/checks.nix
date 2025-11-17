{
  craneLib,
  commonArgs,
  cargoArtifacts,
  formatter,
}: {
  inherit formatter;
  clippy = craneLib.cargoClippy (commonArgs
    // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "--lib --bins -- -D warnings";
    });
}
