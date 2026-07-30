# vipdf

A minimal, PDF viewer written in Rust with vim keybindings.
PDF pages are rendered on the CPU and displayed in a native window.

## Requirements

- Rust **1.85 or newer** (the project uses the 2024 edition). The included Nix
  flake pins Rust 1.96.1, which satisfies this.
- Linux:
    - NixOS or Nix package manager: A flake for building the project is available.
    - Other distros: If you are on Wayland, you'll need the usual runtime libraries (`libxkbcommon`, `wayland`,
      `libGL`).

## Building & running

Please build in release mode for acceptable performance.

On NixOS just clone the repo and run
```shell script
nix develop
nix build
./result/bin/vipdf path/to/file.pdf
```

Otherwise using cargo:

```shell script
cargo run --release -- path/to/file.pdf
```

Or even better, build a binary and run it directly:

```shell script
cargo build --release
./target/release/vipdf path/to/file.pdf
```

## Usage

| Key           | Action                                                                     |
|---------------|----------------------------------------------------------------------------|
| `gg`          | **Top of the first page**                                                  |
| `Shift+g`     | **Bottom of the last page**                                                |
| `j`           | **Next page** or **scroll down** if page is taller than window             |
| `k`           | **Previous page** or **scroll up** if page is taller than the window     |
| `${number}gg` | **Jump to page ${number}** (Careful: might not match table of content) |
| `+`           | Zoom in                                                                    |
| `-`           | Zoom out                                                                   |
| Close window  | Quit                                                                       |

When a page fits entirely within the window, `j` / `k` turn pages. When a page
is taller than the window (after zooming in), `j` / `k` scroll
within the page first, and only turn the page once you reach the bottom/top.

>**Hint** If you want to scroll many pages fast just zoom out!

## Project status & roadmap

This is an actively-changing project. Things that are planned:

- Horizontal scrolling
- Search mode

## AI usage

All AI involvement happened with JetBrains AI
Assistant, used purely as a conversational assistant. It had **no agentic
capabilities** it could not run, build, test, edit files autonomously.
Every suggestion was reviewed, corrected, and integrated by the
author.

Weighted by intellectual contribution and effort, the split
is approximately **85% author / 15% AI**. The AI's share is concentrated in
`winit`/`softbuffer` windowing and pixel-blit boilerplate, plus some bug
diagnoses.
