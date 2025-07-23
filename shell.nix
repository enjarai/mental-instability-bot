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

  RUST_BACKTRACE = 1;
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libs;
}
