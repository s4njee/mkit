# Install

## Rust and the pinned GPUI package

Install Rust and Cargo through [rustup](https://rust-lang.org/tools/install/). This workspace declares Rust **1.92** as its minimum supported version and uses edition 2024. CI installs the current stable toolchain; that is its tested setup, not a claim that every older compiler above the minimum was tested. Check your installation with `rustc --version` and `cargo --version`.

The workspace pins the crates.io package **`gpui-pre` to `=0.3.5`**. Its source is Zed revision [`d89e9c2`](https://github.com/zed-industries/zed/tree/d89e9c2124b2786a390c7a451c7488601b4da2e1). Cargo renames the package to the dependency key `gpui_pre`, so examples import `gpui_pre`. The separate crates.io package named `gpui` is a different release line; substituting it can give you incompatible GPUI types. GPUI Kit 0.6.4 also depends on `gpui-pre` 0.3.5. See [Versions](appendices/versions.md) for the exact manifest and compatibility rule. The [published `gpui-pre` 0.3.5 guide](https://docs.rs/crate/gpui-pre/0.3.5) describes its platform backends.

## Operating system prerequisites

These are the native tools needed to build or run GPUI, followed by the packages this repository installs in CI. The [Rust installation guide](https://rust-lang.org/tools/install/) explains the compiler toolchain; the [pinned GPUI guide](https://docs.rs/crate/gpui-pre/0.3.5) explains its platform backends.

| Platform | Prepare the host |
| --- | --- |
| macOS | Install Xcode, launch it once to install its components, and install the Xcode Command Line Tools with `xcode-select --install`. GPUI's renderer uses Metal. If several Xcode installations exist, select the intended developer directory with `xcode-select`. |
| Windows | Use a Rust MSVC toolchain and install Visual Studio C++ Build Tools with a Windows SDK. Rust's installer may prompt for these tools. GPUI uses Win32 and DirectWrite here. |
| Linux | Install a C/C++ toolchain, `pkg-config`, and the native development libraries for the Wayland or X11 backend you build. Package names differ by distribution. The Ubuntu CI list below is a tested starting point for this repository; [Zed's Linux build guide](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/docs/src/development/linux.md) covers the larger editor and may need more packages. |

CI runs on `ubuntu-latest`, `macos-latest`, and `windows-latest`. Its Ubuntu job installs this exact list from this repository's `.github/workflows/ci.yml`:

```sh
sudo apt-get install -y --no-install-recommends \
  build-essential cmake clang lld llvm pkg-config \
  libasound2-dev libfontconfig-dev libgit2-dev libglib2.0-dev \
  libssl-dev libva-dev libvulkan1 libwayland-dev \
  libx11-xcb-dev libxkbcommon-x11-dev libzstd-dev libsqlite3-dev
```

This CI list records one runner setup. It is not a minimal dependency declaration for every Linux distribution or feature set. The pinned GPUI guide says Linux desktop windows need at least one of its Wayland or X11 features; this workspace's resolved features come from its manifests and lockfile.

## Build and check

From the repository root, run:

```sh
cargo build --workspace --locked
cargo test --workspace --locked
```

To build this book, install [mdBook](https://rust-lang.github.io/mdBook/guide/installation.html) and Node.js for the version banner preprocessor. CI uses mdBook **0.4.40**, Python **3.12**, and codespell **2.4.3** for book checks. Then run:

```sh
python3 scripts/check_book.py
mdbook build book
```

On Windows, use `py -3` if `python3` is not on your path. Add `--spell` to the checker after installing codespell. The [hello chapter](examples/hello.md) has the first runnable window and a screenshot test.

## Screenshot coverage

The headless screenshot baseline runs on macOS with its Metal renderer:

```sh
cargo test -p mkit-example-hello --test screenshot --locked
```

Windows and Linux compile the example, but the pinned GPUI release does not expose a headless screenshot renderer there; the test reports the unsupported capture. The repository's `docs/spikes/headless.md` records the observed platform behavior. To regenerate the book image after an intentional visual change, use `UPDATE_SNAPSHOTS=1` with the command above on macOS and inspect the new image before review.
