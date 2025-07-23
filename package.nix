# lib isn't necessary if there's no license
{ craneLib, lib, openssl, pkgconf }:

craneLib.buildPackage {
  src = craneLib.cleanCargoSource ./.;

  buildInputs = [
    openssl
  ];

  nativeBuildInputs = [
    pkgconf
  ];

  doCheck = false;

  meta = with lib; {
    homepage = "https://github.com/enjarai/mental-instability-bot";
    description = "The bot of the discord";
  };
}
