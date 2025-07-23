# lib isn't necessary if no license is defined
{ lib, makeRustPlatform, rust-bin, openssl, pkgconf }:

(makeRustPlatform {
  cargo = rust-bin.nightly.latest.default;
  rustc = rust-bin.nightly.latest.default;
}).buildRustPackage rec {
  pname = "mental-instability-bot";
  version = "0.1.0";

  src = ./.;

  buildInputs = [
    openssl
  ];

  nativeBuildInputs = [
    pkgconf
  ];

  cargoLock = {
    lockFile = src + /Cargo.lock;

    outputHashes = {
      "poise-0.6.1" = "sha256-j72ha9Sn/A8F/PGoPDCAF8ThlPuigUWpK1GoFJSvPxg=";
      "serenity-0.12.1" = "sha256-g2/5dP8gDwYybhbG9iD59xwOKaKKDDlkYEerlhDYd9A=";
    };
  };

  doCheck = false;

  meta = with lib; {
    homepage = "https://github.com/enjarai/mental-instability-bot";
    description = "The bot of the discord";
  };
}
