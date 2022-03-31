with import <nixpkgs> {};

mkShell {
  buildInputs = [
    cargo
    openssl
    pkg-config
    rust-analyzer
    rustc
    rustfmt
  ];


  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  RUSTC_BOOTSTRAP = "1";
}
