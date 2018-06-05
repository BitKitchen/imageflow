let
  pkgs = import <nixpkgs> {};
  defaultRustChannel = pkgs.rustChannelOf { date = "2018-05-15"; channel = "nightly"; };
  defaultRust = pkgs.rust;
  defaultCargo = pkgs.cargo;

  fetchCargo = pkgs.callPackage (import <nixpkgs/pkgs/build-support/rust/fetchcargo.nix>) {
    rust = defaultRustChannel.rust;
  };
  buildRustCrate = pkgs.callPackage (import <nixpkgs/pkgs/build-support/rust/build-rust-crate.nix>) {
    rustc = defaultRustChannel.rust;
  };
  imageflow_pkgs = (pkgs.callPackage ./Cargo.nix {
  inherit buildRustCrate;
});
#
in rec {
  mozjpeg_sys_0_5_13 = (imageflow_pkgs.mozjpeg_sys_0_5_13 {}).override {
    crateOverrides = pkgs.defaultCrateOverrides // {
      mozjpeg-sys = attrs: {
        #type = "cdylib";
        buildInputs = [ pkgs.zlib pkgs.libjpeg ];
      };
    };
  };
  libpng_sys_0_2_6 = (imageflow_pkgs.libpng_sys_0_2_6 {}).override {
    buildInputs = [ pkgs.zlib pkgs.libpng ];
    crateOverrides = pkgs.defaultCrateOverrides // {
      libpng-sys = attrs: {
        buildInputs = [ pkgs.zlib pkgs.libjpeg ];
        postInstall = "ls -alhR target; ls -alhR $out";
      };
    };
  };
  lcms2_sys_2_4_8 = (imageflow_pkgs.lcms2_sys_2_4_8 {}).override {
    crateOverrides = pkgs.defaultCrateOverrides // {
      lcms2-sys = attrs: {
        buildInputs = [ pkgs.zlib ];
      };
    };
  };
  imageflow_server = (imageflow_pkgs.imageflow_server {}).override {
    crateOverrides = pkgs.defaultCrateOverrides // {
      mozjpeg-sys = attrs: {
        buildInputs = [ pkgs.zlib pkgs.libjpeg ];
      };
      libpng-sys = attrs: {
        buildInputs = [ pkgs.zlib pkgs.libpng ];
      };
      imageflow_c_components = attrs: {
        type = "staticlib";
        src = ./c_components;
        buildInputs = [ pkgs.zlib pkgs.libjpeg ];
        preConfigure = ''
            #export DEP_JPEG_INCLUDE=${mozjpeg_sys_0_5_13}/lib/mozjpeg-sys.out/include/
            export DEP_JPEG_INCLUDE=${pkgs.libjpeg.dev}/include
            export DEP_PNG_INCLUDE=${pkgs.libpng.dev}/include
            export DEP_LCMS2_INCLUDE_ALT=${pkgs.lcms2.dev}/include
            export RUST_LOG=debug
          '';
      };
    };
  };
}

