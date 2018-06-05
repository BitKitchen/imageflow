let
  pkgs = import <nixpkgs> {};
  cargo-vendor = (pkgs.callPackage (import ./cargo-vendor-deps.nix) {}).cargo_vendor {};
  buildRustPackage = pkgs.callPackage (import <nixpkgs/build-support/rust>) {
    #inherit cargo-vendor;
  };
in {
  imageflow = pkgs.callPackage ./derivation.nix {
    inherit buildRustPackage;
  };
  inherit cargo-vendor;
}
