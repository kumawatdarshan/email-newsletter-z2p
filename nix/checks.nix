{
  craneLib,
  commonArgs,
  cargoArtifacts,
}: {
  clippy = craneLib.cargoClippy (commonArgs
    // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "--lib --bins -- -D warnings";
    });
}
