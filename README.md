# vipdf

A minimal, PDF viewer written in Rust with vim keybindings. PDF pages are rendered on the CPU and displayed in a native
window.

## Rendering backends

vipdf supports two rendering backends, selected at build time:

- **hayro** (default) - a pure-Rust PDF renderer. No system libraries needed beyond the usual windowing ones. _hayro is
  experimental but already very stable._ In case you run into any unforeseen issues, there is still pdfium as a
  fallback.
- **pdfium** (optional, cargo feature `pdfium`) - Google's PDF library, as used in Chromium. Generally faster and more
  complete, but requires the native
  `libpdfium.so` shared library to be present on your system at **runtime**
  (it is loaded dynamically; nothing is needed at build time).

## Requirements

- Rust **1.85 or newer**. The included Nix flake pins Rust 1.96.1, which satisfies
  this.
- Linux:
    - NixOS or Nix package manager: A flake for building the project is available (for both versions).
    - Other distros: If you are on Wayland, you'll need the usual runtime libraries (`libxkbcommon`, `wayland`,
      `libGL`).
    - For the optional pdfium backend: `libpdfium.so` (see below).

## Building & running

Please build in release mode for acceptable performance. Also make sure to alias the build (in case you compiled it) to
`vipdf`.

### NixOS

On NixOS clone the repo and run:

```shell script
nix develop
nix build
./result/bin/vipdf path/to/file.pdf
```

to build with the pdfium backend instead:

```shell script
nix build .#pdfium
./result/bin/vipdf path/to/file.pdf
```

or run it directly without keeping a result symlink:

```shell script
nix run .#pdfium -- path/to/file.pdf
```

### Other distros

#### Build and run

Just run with:

```shell script
cargo run --release -- path/to/file.pdf
```

Or even better, build a binary and run it directly:

```shell script
cargo build --release
./target/release/vipdf path/to/file.pdf
```

#### pdfium support

for pdfium support just add `--features pdfium` so e.g.

```shell script
cargo build --release --features pdfium
./target/release/vipdf path/to/file.pdf
```

For the pdfium backend, `libpdfium.so` must be findable by the dynamic loader at runtime:

#### How to get pdfium

_CachyOS / Arch-based distros_: install a pdfium package from the AUR, e.g. `pdfium-binaries-bin` (or
  `libpdfium-nojs`):

  ```shell script
  paru -S pdfium-binaries-bin
  ```

_Debian / Ubuntu_: There is no official apt package for standalone pdfium... You can download a prebuilt release from
  [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries/releases)
  and add it to your `LD_LIBRARY_PATH`:

  ```shell script
  export LD_LIBRARY_PATH=/path/to/dir/containing/libpdfium:$LD_LIBRARY_PATH
  ./target/release/vipdf path/to/file.pdf
  ```
  
  >Honestly, don't do this is, it's ugly... use vipdf default instead or consider switching to NixOS.

## Usage

let `N` be a natural number:

| Key          | Action                                                                                           |
|--------------|--------------------------------------------------------------------------------------------------|
| `gg`         | **Top of the first page**                                                                        |
| `Ngg`        | **Jump to page `N`** (Careful: might not match table of content)                                 |
| `Shift+g`    | **Bottom of the last page**                                                                      |
| `j`          | **Next page** or **scroll down** if page is taller than window                                   |
| `Nj`         | **Execute `N` times next page** or **scroll down `N` times** if page is taller than window       |
| `k`          | **Previous page** or **scroll up** if page is taller than the window                             |
| `Nk`         | **Execute `N` times previous page** or **scroll up `N` times** if page is taller than the window |
| `+`          | Zoom in                                                                                          |
| `-`          | Zoom out                                                                                         |
| Close window | Quit                                                                                             |

When a page fits entirely within the window, `j` / `k` turn pages. When a page is taller than the window (after zooming
in), `j` / `k` scroll within the page first, and only turn the page once you reach the bottom/top.

> **Hint** If you want to scroll many pages fast just zoom out!

## Project status & roadmap

This is an actively changing project. Things that are planned:

- Horizontal scrolling
- Search mode

## AI usage

All AI involvement happened with JetBrains AI Assistant, used purely as a conversational assistant. It had **no agentic
capabilities** it could not run, build, test, edit files autonomously. Every suggestion was reviewed, corrected, and
integrated by the author.

Weighted by intellectual contribution and effort, the split is approximately **85% author / 15% AI**. The AI's share is
concentrated in
`winit`/`softbuffer` windowing and pixel-blit boilerplate, plus some bug diagnoses.
