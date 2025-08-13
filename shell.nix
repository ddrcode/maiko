{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  packages = with pkgs; [
    cargo
    clippy
    rust-analyzer
    rustc
    rustfmt
    treefmt
    openssl
    pkg-config
    cargo-machete
  ];

  # inputsFrom = [ pkgs.hello pkgs.gnutar ];

  shellHook = ''
    export DEBUG=1
  '';
}
