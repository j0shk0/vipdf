{
  description = "vipdf - a minimal PDF viewer";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable."1.96.1".default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        runtimeLibs = with pkgs; [
          libGL
          libxkbcommon
          wayland
          libX11
          libXcursor
          libXrandr
          libXi
          libxcb
        ];

        libPath = pkgs.lib.makeLibraryPath runtimeLibs;

        commonArgs = {
          pname = "vipdf";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = runtimeLibs ++ [ pkgs.stdenv.cc.cc.lib ];
          meta = with pkgs.lib; {
            description = "A minimal PDF viewer written in Rust with vim keybindings";
            license = licenses.gpl3Only;
            mainProgram = "vipdf";
            platforms = platforms.linux;
          };
        };

        vipdf = rustPlatform.buildRustPackage (commonArgs // {
          nativeBuildInputs = with pkgs; [ pkg-config autoPatchelfHook ];
          runtimeDependencies = runtimeLibs;
        });
      in
      {
        packages = {
          default = vipdf;
        };

        apps.default = {
          type = "app";
          program = "${vipdf}/bin/vipdf";
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ rustToolchain ];
          buildInputs = with pkgs; [ cargo-about pkg-config rust-analyzer ] ++ runtimeLibs;
          LD_LIBRARY_PATH = libPath;
          shellHook = ''
            mkdir -p ~/.rust-rover/toolchain
            ln -sfn ${rustToolchain}/lib ~/.rust-rover/toolchain
            ln -sfn ${rustToolchain}/bin ~/.rust-rover/toolchain
            export RUST_SRC_PATH="$HOME/.rust-rover/toolchain/lib/rustlib/src/rust/library"
          '';
        };
      }
    );
}
