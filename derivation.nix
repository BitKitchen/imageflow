{ buildRustPackage, openssl, pkgconfig, fetchurl, libpng, libjpeg }:

buildRustPackage rec {
  name = "imageflow-${version}";
  version = "blah";

  src = ./.;

  # This needs to be fetched from a failed nix-build
  cargoSha256 = "0qhn0v9sss1rlzk4nq77lrl7b7lmsn5gywg8j52345a11ks3h9zp";

  buildInputs = [ openssl libpng libjpeg ];
  nativeBuildInputs = [ pkgconfig ];
  enableParallelBuilding = true;
}
