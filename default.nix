let
  defaultPkgs = import <nixpkgs> {};
in

{
  openssl ? defaultPkgs.openssl,
  pkg-config ? defaultPkgs.pkg-config,
  rustPlatform ? defaultPkgs.rustPlatform,
}:

rustPlatform.buildRustPackage rec {
  pname = "entman";
  version = "unstable";

  meta = with pkgs.lib; {
    platforms = platforms.linux;
  };

  src = ./.;

  cargoSha256 = "0affpkd4aqyymgzazka697fyhmwscdb0i6vxnbm1f4pfb0136dkc";

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
  ];

  RUSTC_BOOTSTRAP = "1";
}
