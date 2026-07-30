# vipdf

A minimal, PDF viewer written in Rust with vim keybindings (currenctly implementing only `j`, `k`, `gg`, `Shift+g`).
PDF pages are rendered on the CPU and displayed in a native window.

> **This project is under development.**
> Expect rough edges and missing features. Of course, if you can't resist, feel free to try it out.

## Requirements

- Rust **1.85 or newer** (the project uses the 2024 edition). The included Nix
  flake pins Rust 1.96.1, which satisfies this.
- Linux:
    - NixOS or Nix package manager: There is a developer shell available. A package might come when the project status
      is
      acceptable.
    - Other distros: if you are on Wayland, you'll need the usual runtime libraries (`libxkbcommon`, `wayland`,
      `libGL`).

## Building & running

Please build in release mode for acceptable performance.

```shell script
cargo run --release -- path/to/file.pdf
```

Or even better, build a binary and run it directly:

```shell script
cargo build --release
./target/release/vipdf path/to/file.pdf
```

## Controls

| Key          | Action                                                                 |
|--------------|------------------------------------------------------------------------|
| `gg`         | **Top of the first page**                                              |
| `Shift+g`    | **Bottom of the last page**                                            |
| `j`          | **Next page** or **scroll down** if page is taller than window         |
| `k`          | **Previous page** or **scroll **up** if page is taller than the window |
| `+`          | Zoom in                                                                |
| `-`          | Zoom out                                                               |
| Close window | Quit                                                                   |

When a page fits entirely within the window, `j` / `k` turn pages. When a page
is taller than the window (after zooming in), `j` / `k` scroll
within the page first, and only turn the page once you reach the bottom/top.

## Project status & roadmap

This is an actively-changing project. Things that are known-rough or
planned:

- Rendering all pages up front (high memory use on large documents); lazy,
  cached per-page rendering is planned.
- No horizontal scrolling yet.
- Keybindings are still evolving.
- Error handling is currently minimal in places.

## AI usage

All AI involvement happened within a single chat session with JetBrains AI
Assistant, used purely as a conversational assistant. It had **no agentic
capabilities** it could not run, build, test, edit files autonomously.
Every suggestion was reviewed, corrected, and integrated by the
author.

Weighted by intellectual contribution and effort, the split
is approximately **75% author / 25% AI**. The AI's share is concentrated in
`winit`/`softbuffer` windowing and pixel-blit boilerplate, plus some bug
diagnoses.
