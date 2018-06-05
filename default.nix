let
  pkgs = import <nixpkgs> {};
  rustChannel = pkgs.rustChannelOf { date = "2018-05-15"; channel = "nightly"; };
  rust = rustChannel.rust;
  cargo = rustChannel.cargo;
  buildRustCrate = pkgs.callPackage (import <nixpkgs/pkgs/build-support/rust/build-rust-crate.nix>) {
    rustc = rustChannel.rust;
  };
  cargo-vendor = ((pkgs.callPackage ./cargo-vendor-deps.nix { inherit buildRustCrate; }).cargo_vendor_0_1_15 {}).overrideAttrs (attrs: {
    src = pkgs.fetchFromGitHub {
      owner = "alexcrichton";
      repo = "cargo-vendor";
      rev = "0.1.15";
      sha256 = "0jbywx7111v0pqs1c4b88pwgqy0ycdym7fyy09ywwazbvcpki0rr";
    };
  });
  fetchcargo = pkgs.callPackage (import <nixpkgs/pkgs/build-support/rust/fetchcargo.nix>) {
    inherit cargo-vendor;
  };
  buildRustPackage = pkgs.callPackage (import <nixpkgs/pkgs/build-support/rust/default.nix>) {
    inherit cargo-vendor;
    rust = { inherit (rustChannel) rustc cargo; };
  };
in {
  imageflow = pkgs.callPackage ./derivation.nix {
    inherit buildRustPackage fetchcargo rust cargo;
  };
  shell = pkgs.stdenv.mkDerivation {
    name = "imageflow-shell";
    buildInputs = [ rust cargo ];
  };
  inherit cargo-vendor rust cargo;
}
