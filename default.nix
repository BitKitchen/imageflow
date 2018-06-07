let
  pkgs = import <nixpkgs> {};
  rustChannel = pkgs.rustChannelOf { date = "2018-05-15"; channel = "nightly"; };
  rust = rustChannel.rust;
  cargo = rustChannel.cargo;
  buildRustCrate = pkgs.callPackage (import ./build-rust-crate.nix) {
    rustc = rust;
  };
  buildRustCrateHelpers = pkgs.callPackage ./build-rust-crate-helpers.nix { };
in (pkgs.callPackage ./Cargo.nix {
  inherit buildRustCrate buildRustCrateHelpers;
}).imageflow_server."0.1.0".override {
  crateOverrides = pkgs.defaultCrateOverrides // {
    libpng-sys = attrs: {
      buildInputs = [ pkgs.zlib ];
    };
    imageflow_c_components = attrs: {
      buildInputs = [ pkgs.zlib ];
    };
  };
}
