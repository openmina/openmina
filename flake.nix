{
  description = "Mina Rust prerequisites";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustup
            rust-analyzer
            libiconv # Needed for macOS for rustup

            openssl
            jemalloc
            protobuf
            libpcap

            pkg-config
            gcc
            gnumake

            curl
            git
            nodejs

            shellcheck
          ];

          # To use wasm-pack and other executables installed via `cargo install`
          shellHook = ''
            export PATH=$HOME/.cargo/bin:$PATH
          '';

          # Fix for tikv-jemalloc-sys compilation error on GNU platforms:
          # The GNU version of strerror_r returns char* while POSIX returns int.
          # Only set this flag on Linux/GNU systems, not needed on macOS.
          CFLAGS = if pkgs.stdenv.isLinux then "-DJEMALLOC_STRERROR_R_RETURNS_CHAR_WITH_GNU_SOURCE" else "";
          JEMALLOC_SYS_WITH_CFLAGS = if pkgs.stdenv.isLinux then "-DJEMALLOC_STRERROR_R_RETURNS_CHAR_WITH_GNU_SOURCE" else "";
        };
      }
    );
}
