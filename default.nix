let
  defaultPkgs = import <nixpkgs> {};
in

{
  fetchFromGitHub ? defaultPkgs.fetchFromGitHub,
  makeRustPlatform ? defaultPkgs.makeRustPlatform,
  openssl ? defaultPkgs.openssl,
  pkg-config ? defaultPkgs.pkg-config,
  pkgs ? defaultPkgs
}:

let
  mozRepo = fetchFromGitHub {
    owner = "mozilla";
    repo = "nixpkgs-mozilla";
    rev = "50bae918794d3c283aeb335b209efd71e75e3954";
    sha256 = "07b7hgq5awhddcii88y43d38lncqq9c8b2px4p93r5l7z0phv89d";
  };
  moz = import "${mozRepo}/package-set.nix" { inherit pkgs; };
  nightlyChannel = moz.latest.rustChannels.nightly;
  nightlyRustPlatform = makeRustPlatform {
    rustc = nightlyChannel.rust;
    cargo = nightlyChannel.cargo;
  };
in

nightlyRustPlatform.buildRustPackage rec {
  name = "entman-${version}";
  version = "unstable";

  src = ./.;

  cargoSha256 = "08mbzf6ryyjzhwzk5hri8x17j2x783pjmawh13vc0qgvrfwav6jm";

  preConfigure = ''
    export HOME=$(mktemp -d)
  '';

  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    openssl
  ];
}
