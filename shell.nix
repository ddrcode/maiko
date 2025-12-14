{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  packages = with pkgs; [
    bacon
    cargo
    clippy
    rust-analyzer
    rustc
    rustfmt
    cargo-machete
    cargo-spellcheck
    cargo-readme
    hunspell
    just
  ];

  # inputsFrom = [ pkgs.hello pkgs.gnutar ];

  shellHook = ''
    export DEBUG=1
  '';
}
