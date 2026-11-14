{ lib
, stdenv
, rustPlatform
, fetchFromGitHub
, pkg-config
, autoPatchelfHook
, makeWrapper
, libGL
, libxkbcommon
, wayland
, libX11
, libXcursor
, libXrandr
, libXi
, libxcb
, pdfium-binaries
, withPdfium ? false
}:

let
  # winit loads these with dlopen at runtime, so they must end up in the RPATH
  runtimeLibs = [
    libGL
    libxkbcommon
    wayland
    libX11
    libXcursor
    libXrandr
    libXi
    libxcb
  ];
in
rustPlatform.buildRustPackage {
  pname = if withPdfium then "vipdf-pdfium" else "vipdf";
  version = "0.2.0-unstable-2026-11-05";

  src = fetchFromGitHub {
    owner = "j0shk0";
    repo = "vipdf";
    rev = "01bd9db379b9bee23d522e4fa54f2f5e79b60cfb";
    hash = "sha256-A1iiJ20v2u8O9aOV8Dkztf9MPTFmMKxyR6SXszOOXZY=";
  };

  cargoHash = "sha256-yICcxZvbB4F4ibk0w5Mof6qibE/f9bG/gA5IloaYRes=";

  nativeBuildInputs = [ pkg-config autoPatchelfHook ]
    ++ lib.optionals withPdfium [ makeWrapper ];

  buildInputs = runtimeLibs ++ [ stdenv.cc.cc.lib ];

  runtimeDependencies = runtimeLibs
    ++ lib.optionals withPdfium [ pdfium-binaries ];

  buildFeatures = lib.optionals withPdfium [ "pdfium" ];

  postFixup = lib.optionalString withPdfium ''
    wrapProgram $out/bin/vipdf \
      --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [ pdfium-binaries ]}
  '';

  meta = with lib; {
    description = "A minimal PDF viewer written in Rust with vim keybindings";
    homepage = "https://github.com/j0shk0/vipdf";
    license = licenses.gpl3Only;
    mainProgram = "vipdf";
    platforms = platforms.linux;
  };
}
