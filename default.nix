let
  pkgs = import <nixpkgs> {};
  rustChannel = pkgs.rustChannelOf { date = "2018-05-15"; channel = "nightly"; };
  rust = rustChannel.rust;
  cargo = rustChannel.cargo;
  buildRustCrate = pkgs.callPackage (import ./build-rust-crate.nix) {
    rustc = rust;
  };
  buildRustCrateHelpers = pkgs.callPackage ./build-rust-crate-helpers.nix { };
  mozjpegsyssrc = pkgs.fetchgit {
    url = "https://github.com/kornelski/mozjpeg-sys.git";
    rev = "384688f9c23e94ddeb353d414d45ede69768ec08";
    sha256 = "0ln6y6mrddv2gi9l4nmqpvllhanccz6cp22y66s6m1kqh2gh16ka";
    fetchSubmodules = true;
  };
  mozjpegsrc = pkgs.fetchgit {
    url = "https://github.com/mozilla/mozjpeg.git";
    rev = "365bc1ce1197914ef21af9673c7a5d06e40fc2a1";
    sha256 = "0395i1x72mmfg00y4njhs53kwx831w0vlv8s6ch4y5lpla9lkvjf";
  };
in (pkgs.callPackage ./Cargo.nix {
  inherit buildRustCrate buildRustCrateHelpers;
}).imageflow_server."0.1.0".override {
  crateOverrides = pkgs.defaultCrateOverrides // rec {
    libpng-sys = attrs: {
      buildInputs = [ pkgs.zlib ];
    };
    macro_attr = attrs: {
      src = pkgs.fetchgit {
        url = "https://github.com/DanielKeep/rust-custom-derive.git";
        rev = "1252f258cdb9b7c9867f937c52c2f5c0e69a9c03";
        sha256 = "0hkigymvxdrd1zjkqyg1gscjwhi2fbyfgi0pzl1rn0pg4gpdij8d";
      };
    };
    macro-attr = macro_attr;
    enum_derive = attrs: {
      src = pkgs.fetchgit {
        url = "https://github.com/DanielKeep/rust-custom-derive.git";
        rev = "1252f258cdb9b7c9867f937c52c2f5c0e69a9c03";
        sha256 = "0hkigymvxdrd1zjkqyg1gscjwhi2fbyfgi0pzl1rn0pg4gpdij8d";
      } + "/enum_derive";
    };
    enum-derive = enum_derive;
    imageflow_c_components = attrs: {
      buildInputs = [ pkgs.zlib pkgs.libjpeg.dev pkgs.libpng.dev pkgs.lcms2.dev mozjpegsrc ];
      preConfigure = ''
          export DEP_JPEG_INCLUDE="${pkgs.libjpeg.dev}/include:${mozjpegsrc}"
          export DEP_PNG_INCLUDE=${pkgs.libpng.dev}/include
          export DEP_LCMS2_INCLUDE_ALT=${pkgs.lcms2.dev}/include
        '';
    };
    imageflow_types = attrs: {
      nativeBuildInputs = [ pkgs.git ];
    };
    imageflow_server = attrs: {
      nativeBuildInputs = [ pkgs.zlib ];
    };
  };
}
