{ craneLib, lib, rustPlatform, openssl, pkgconf }:

craneLib.buildPackage {
  src = craneLib.cleanCargoSource ./.;

  buildInputs = [
    openssl
  ];

  nativeBuildInputs = [
    pkgconf
  ];

  doCheck = false;
}
# rustPlatform.buildRustPackage rec {
#   pname = "mental-instability-bot";
#   version = "0.1.0";

#   src = ./.;

#   buildInputs = [
#     # alsa-lib
#     # openssl
#   ];

#   nativeBuildInputs = [
#     pkgconf
#   ];

#   cargoLock = {
#     lockFile = src + /Cargo.lock;
#     allowBuiltinFetchGit = true;
#   };
#   doCheck = false;

#   meta = with lib; {
#     homepage = "https://github.com/enjarai/mental-instability-bot";
#     description = "The bot of the discord";
#   };
# }