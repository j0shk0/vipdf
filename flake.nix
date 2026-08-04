{
  description = "vipdf - a minimal PDF viewer";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
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
        libPath = pkgs.lib.makeLibraryPath (runtimeLibs ++ [ pkgs.pdfium-binaries ]);
        commonArgs = {
          pname = "vipdf";
          version = "0.2.0";
          src = pkgs.lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = runtimeLibs ++ [ pkgs.stdenv.cc.cc.lib ];
          meta = with pkgs.lib; {
            description = "A minimal PDF viewer written in Rust with vim keybindings";
            license = licenses.gpl3Only;
            mainProgram = "vipdf";
            platforms = platforms.linux;
          };
        };
        mkVipdf = { withPdfium ? false }:
          rustPlatform.buildRustPackage (commonArgs // {
            pname = if withPdfium then "vipdf-pdfium" else "vipdf";
            nativeBuildInputs = with pkgs; [ pkg-config autoPatchelfHook ]
              ++ pkgs.lib.optionals withPdfium [ pkgs.makeWrapper ];
            runtimeDependencies = runtimeLibs
              ++ pkgs.lib.optionals withPdfium [ pkgs.pdfium-binaries ];
            buildFeatures = pkgs.lib.optionals withPdfium [ "pdfium" ];
            postFixup = pkgs.lib.optionalString withPdfium ''
              wrapProgram $out/bin/vipdf \
                --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath [ pkgs.pdfium-binaries ]}
            '';
          });
        vipdf = mkVipdf { };
        vipdf-pdfium = mkVipdf { withPdfium = true; };
      in
      {
        packages = {
          default = vipdf;
          pdfium = vipdf-pdfium;
        };
        apps = {
          default = {
            type = "app";
            program = pkgs.lib.getExe vipdf;
          };
          pdfium = {
            type = "app";
            program = pkgs.lib.getExe vipdf-pdfium;
          };
        };
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ rustToolchain ];
          buildInputs = with pkgs; [ cargo-about pkg-config rust-analyzer ] ++ runtimeLibs;
          PDFIUM_LIB_DIR = "${pkgs.pdfium-binaries}/lib";
          PDFIUM_INCLUDE_DIR = "${pkgs.pdfium-binaries}/include";
          shellHook = ''
            export LD_LIBRARY_PATH="${libPath}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
            toolchainDir="$HOME/.rust-rover/nix-toolchain"
            if [ "$(cat "$toolchainDir/.stamp" 2>/dev/null)" != "${rustToolchain}" ]; then
              chmod -R u+w "$toolchainDir" 2>/dev/null || true
              rm -rf "$toolchainDir"
              mkdir -p "$toolchainDir"
              cp -rs ${rustToolchain} "$toolchainDir/toolchain"
              echo "${rustToolchain}" > "$toolchainDir/.stamp"
            fi
            export RUST_SRC_PATH="$toolchainDir/toolchain/lib/rustlib/src/rust/library"
          '';
        };
      }
    );
}
