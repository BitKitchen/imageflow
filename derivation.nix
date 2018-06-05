{ buildRustPackage, openssl, pkgconfig, fetchurl, libpng, libjpeg, fetchcargo, rust, cargo, conan, cmake, nasm }:


let
  #cargoVendorDir = fetchcargo {
  #  src = ./.;
  #  srcs = null;
  #  sourceRoot = null;
  #  sha256 = "1h3sbxn893ygnhca1sc720c61ag6s25mbxzxc75g627hw137ayha";
  #};
  cargoVendorDir = null;
  setupVendorDir = if cargoVendorDir == null
    then ''
      unpackFile "$cargoDeps"
      cargoDepsCopy=$(stripHash $(basename $cargoDeps))
      chmod -R +w "$cargoDepsCopy"
    ''
    else ''
      cargoDepsCopy="$sourceRoot/${cargoVendorDir}"
    '';
  logLevel = "warn";
in

buildRustPackage rec {
  name = "imageflow-${version}";
  version = "blah";

  src = ./.;

  # This needs to be fetched from a failed nix-build
  cargoSha256 = "1ryf4m0h7sl4x43mcb8f6522p4b6wab5as47b5ra9z5d71vgypqn";
  inherit cargoVendorDir;

  postUnpack = ''
    eval "$cargoDepsHook"
    ${setupVendorDir}
    mkdir -p .cargo
    cat >.cargo/config <<-EOF
      [source.crates-io]
      registry = 'https://github.com/rust-lang/crates.io-index'
      replace-with = 'vendored-sources'

      [source."https://github.com/DanielKeep/rust-custom-derive.git"]
      git = "https://github.com/DanielKeep/rust-custom-derive.git"
      branch = "master"
      replace-with = "vendored-sources"

      [source."https://github.com/TyOverby/bincode"]
      git = "https://github.com/TyOverby/bincode"
      rev = "0bc25445"
      replace-with = "vendored-sources"

      [source."https://github.com/iron/logger.git"]
      git = "https://github.com/iron/logger.git"
      rev = "0daead5fe10c3cd0c4738767c162dc63a59c3fb3"
      replace-with = "vendored-sources"

      [source."https://github.com/onur/staticfile"]
      git = "https://github.com/onur/staticfile"
      rev = "9f2ff7201eda648128c92e3f5597c587f0629f51"
      replace-with = "vendored-sources"

      [source.vendored-sources]
      directory = '$(pwd)/$cargoDepsCopy'
    EOF
    unset cargoDepsCopy
    export RUST_LOG=${logLevel}
  '';

#  preBuild = ''
#      (cd imageflow_core
#        conan install --build missing -s target_cpu=haswell # Will build imageflow package with your current settings
#      )
#    '';

  buildPhase = with builtins; ''
    runHook preBuild
    echo "Running cargo build --release --frozen"
    HOME=$TMPDIR/fake-home
    cargo --version
    cargo build --release --frozen --verbose
    runHook postBuild
  '';

  buildInputs = [ openssl libpng libjpeg ];
  nativeBuildInputs = [ pkgconfig rust cargo conan cmake nasm ];
  enableParallelBuilding = true;
  doCheck = false;
}
