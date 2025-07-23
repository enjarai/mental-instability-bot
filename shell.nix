{ pkgs ? import <nixpkgs> { } }:

let
  libs = with pkgs; [
    openssl
  ];
in pkgs.mkShell {
  name = "mental-instability-bot";

  buildInputs = libs ++ (with pkgs; [
    rust-bin.nightly.latest.default
    pkgconf
  ]);

  RUST_SRC_PATH = let
      p = with pkgs; (makeRustPlatform {
        cargo = rust-bin.nightly.latest.default;
        rustc = rust-bin.nightly.latest.default;
      }).rustLibSrc;
    in p;
  RUST_BACKTRACE = 1;
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libs;
}
