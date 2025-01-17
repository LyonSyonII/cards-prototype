{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils, ... }@inputs: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ inputs.fenix.overlays.default ];
      };
    in
    {
      devShells.default = pkgs.mkShell rec {
        nativeBuildInputs = with pkgs; [
          sccache
          lld
          mold
          fd
          
          cargo-msrv

          fenix.complete.rustc
          fenix.complete.cargo
          fenix.complete.clippy
          fenix.complete.rustfmt
          fenix.complete.rust-analyzer
          fenix.complete.miri
          fenix.complete.rust-src
          fenix.complete.rustc-codegen-cranelift-preview
          fenix.complete.llvm-tools-preview
          fenix.targets.x86_64-pc-windows-gnu.latest.rust-std
          fenix.targets.x86_64-unknown-linux-gnu.latest.rust-std
  
          curl
          wget
          librsvg
          dbus
          openssl_3
          librsvg
          lzma
          bzip2
          libsoup_3
          pango
          webkitgtk_4_1
          gtk3
          glib
          glibc
          gdk-pixbuf
          cairo
          atkmm
          fribidi
          pkg-config

          xorg.libX11
          xorg.libXcursor

          nodejs_22
          nodePackages.pnpm
        ];
        RUST_SRC_PATH = "${pkgs.fenix.complete.rust-src}/lib/rustlib/src/rust/library";
        RUSTC_WRAPPER="sccache";
        RUSTFLAGS="-Zthreads=12 -Clink-arg=-fuse-ld=mold";
        MSRVFLAGS="-Clink-arg=-fuse-ld=mold"; # RUSTFLAGS=$MSRVFLAGS cargo msrv
        
        LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath nativeBuildInputs;
      };
    });
}
