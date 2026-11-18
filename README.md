<p align="center">
  <img src="res/logo-header.svg" alt="RustDesk - Your remote desktop"><br>
  <a href="#raw-steps-to-build">Build</a> •
  <a href="#how-to-build-with-docker">Docker</a> •
  <a href="#file-structure">Structure</a> •
  <a href="#screenshots">Screenshots</a><br>
  [<a href="docs/README-UA.md">Українська</a>] | [<a href="docs/README-CS.md">česky</a>] | [<a href="docs/README-ZH.md">中文</a>] | [<a href="docs/README-HU.md">Magyar</a>] | [<a href="docs/README-ES.md">Español</a>] | [<a href="docs/README-FA.md">فارسی</a>] | [<a href="docs/README-FR.md">Français</a>] | [<a href="docs/README-DE.md">Deutsch</a>] | [<a href="docs/README-PL.md">Polski</a>] | [<a href="docs/README-ID.md">Indonesian</a>] | [<a href="docs/README-FI.md">Suomi</a>] | [<a href="docs/README-ML.md">മലയാളം</a>] | [<a href="docs/README-JP.md">日本語</a>] | [<a href="docs/README-NL.md">Nederlands</a>] | [<a href="docs/README-IT.md">Italiano</a>] | [<a href="docs/README-RU.md">Русский</a>] | [<a href="docs/README-PTPT.md">Português (Portugal)</a>] | [<a href="docs/README-PTBR.md">Português (Brasil)</a>] | [<a href="docs/README-EO.md">Esperanto</a>] | [<a href="docs/README-KR.md">한국어</a>] | [<a href="docs/README-AR.md">العربي</a>] | [<a href="docs/README-VN.md">Tiếng Việt</a>] | [<a href="docs/README-DA.md">Dansk</a>] | [<a href="docs/README-GR.md">Ελληνικά</a>] | [<a href="docs/README-TR.md">Türkçe</a>] | [<a href="docs/README-NO.md">Norsk</a>] | [<a href="docs/README-RO.md">Română</a>] | [<a href="docs/README-KA.md">ქართული</a>]<br>
  <b>We need your help to translate this README, <a href="https://github.com/rustdesk/rustdesk/tree/master/src/lang">RustDesk UI</a> and <a href="https://github.com/rustdesk/doc.rustdesk.com">RustDesk Doc</a> to your native language</b>
</p>

> [!Caution]
> **Misuse Disclaimer:** <br>
> The developers of RustDesk do not condone or support any unethical or illegal use of this software. Misuse, such as unauthorized access, control or invasion of privacy, is strictly against our guidelines. The authors are not responsible for any misuse of the application.


Chat with us: [Discord](https://discord.gg/nDceKgxnkV) | [Twitter](https://twitter.com/rustdesk) | [Reddit](https://www.reddit.com/r/rustdesk) | [YouTube](https://www.youtube.com/@rustdesk)

[![RustDesk Server Pro](https://img.shields.io/badge/RustDesk%20Server%20Pro-Advanced%20Features-blue)](https://rustdesk.com/pricing.html)

Yet another remote desktop solution, written in Rust. Works out of the box with no configuration required. You have full control of your data, with no concerns about security. You can use our rendezvous/relay server, [set up your own](https://rustdesk.com/server), or [write your own rendezvous/relay server](https://github.com/rustdesk/rustdesk-server-demo).

![image](https://user-images.githubusercontent.com/71636191/171661982-430285f0-2e12-4b1d-9957-4a58e375304d.png)

RustDesk welcomes contribution from everyone. See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for help getting started.

[**FAQ**](https://github.com/rustdesk/rustdesk/wiki/FAQ)

[**BINARY DOWNLOAD**](https://github.com/rustdesk/rustdesk/releases)

[**NIGHTLY BUILD**](https://github.com/rustdesk/rustdesk/releases/tag/nightly)

[<img src="https://f-droid.org/badge/get-it-on.png"
    alt="Get it on F-Droid"
    height="80">](https://f-droid.org/en/packages/com.carriez.flutter_hbb)
[<img src="https://flathub.org/api/badge?svg&locale=en"
    alt="Get it on Flathub"
    height="80">](https://flathub.org/apps/com.rustdesk.RustDesk)

## Dependencies

The native desktop and Android UI is built with [Dioxus](https://dioxuslabs.com/). Reusable controls come from [rust-ui/ui](https://github.com/rust-ui/ui). The default Cargo features select the Dioxus frontend and choose the desktop or Android renderer for the target.

rust-ui uses Tailwind CSS. Run `npm ci` and `npm run build:css` after changing UI classes;
use `npm run watch:css` alongside `dx serve` while developing. The build script resolves
the pinned rust-ui sources through Cargo metadata so its default component classes are
included in `assets/tailwind.css` as well as the application classes.

Desktop uses Dioxus Desktop with the standard WebView renderer (WebKitGTK on Linux, WebView2 on Windows, and WKWebView on macOS). Linux requires `libwebkit2gtk-4.1-dev` and dav1d >= 1.3 (`libdav1d-dev`). AV1 uses low latency dav1d decoding; other frontends retain their existing decoder. On Linux, remote windows use a custom GTK GLArea/OpenGL video surface beneath the transparent WebView. Decoded pixels upload directly from Rust to a reusable GPU texture and return to the decoder buffer pool. Dioxus handles the toolbar, overlays, layout and input; no video pixels cross the WebKit bridge. Windows and macOS retain the binary custom protocol and WebGL canvas path.

Default builds also include hardware codecs. On Linux, install the FFmpeg development packages (`libavcodec-dev libavformat-dev libavutil-dev`) and a VA-API or NVDEC driver for the GPU. Codec backends are detected before a remote session starts. Hardware decoding depends on the negotiated codec: an Intel UHD 630 supports H.264/H.265 through VA-API, while AV1 uses software decoding on that GPU.

## Raw Steps to build

- Prepare your Rust development env and C++ build env

- On Linux, default builds use system media libraries through `pkg-config`; vcpkg is not required.
  Install the development packages listed below before building. FFmpeg remains responsible for
  hardware codecs, libvpx/libaom for software video, libyuv for pixel conversion, and libopus for audio.
  The WebView renderer also requires the platform WebView development libraries.

- On Windows/macOS, install [vcpkg](https://github.com/microsoft/vcpkg) and set `VCPKG_ROOT`
  explicitly. Windows uses `libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static aom:x64-windows-static`;
  macOS uses `libvpx libyuv opus aom`. Hardware builds also need the FFmpeg ports in `vcpkg.json`.
  Apple Silicon can use the existing Homebrew fallback for the software media libraries.

- run `cargo run` to launch the default Dioxus UI
- install the matching Dioxus CLI with `cargo install dioxus-cli --version 0.7.10 --locked`, then run `dx serve --renderer webview` for desktop development

If `dx serve` fails to create its file watcher with `Too many open files`, check
`cat /proc/sys/fs/inotify/max_user_instances`. Linux desktop applications and
editors share this per-user limit; exhausting it also returns this error even
when the process file-descriptor limit is sufficient. To raise the instance limit
for the current boot and future boots:

```sh
sudo install -m 644 res/60-rustdesk-dev-inotify.conf /etc/sysctl.d/60-rustdesk-dev-inotify.conf
sudo sysctl -p /etc/sysctl.d/60-rustdesk-dev-inotify.conf
```

Then retry `dx serve --renderer webview`. Dioxus CLI 0.7.10 creates a watcher even
with `--watch false`; that option does not bypass an exhausted instance limit.

## [Build](https://rustdesk.com/docs/en/dev/build/)

## How to Build on Linux

### Ubuntu 24.04 / Debian 13

```sh
sudo apt install -y build-essential git curl cmake pkg-config clang libclang-dev \
        libgtk-3-dev libwebkit2gtk-4.1-dev libdav1d-dev libvpx-dev libyuv-dev libopus-dev libaom-dev \
        libavcodec-dev libavformat-dev libavutil-dev libva-dev libvdpau-dev \
        libxcb-randr0-dev libxdo-dev libxfixes-dev libxcb-shape0-dev libxcb-xfixes0-dev \
        libasound2-dev libpulse-dev libssl-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev
```

Debian/Ubuntu's libyuv package has no `libyuv.pc`; the build links `libyuv.so` using
its standard header and library paths. Debian packaging also needs `dpkg-dev` to
measure the binary's shared-library dependencies rather than guess versioned FFmpeg package names.

### openSUSE Tumbleweed

Install the matching development packages for GTK3, Fontconfig, dav1d >= 1.3,
libvpx, libyuv, Opus, AOM, FFmpeg, PulseAudio, ALSA, GStreamer, XCB, Xfixes and libxdo,
plus a Rust toolchain, C++ compiler, Clang/libclang and pkg-config. FFmpeg hardware
codec availability depends on the distribution's multimedia repository and GPU driver.

### Fedora

Install the same media and platform development libraries using DNF. Use the
multimedia repository appropriate for your FFmpeg hardware codec support.

### Arch (Manjaro)

```sh
sudo pacman -Syu --needed base-devel git cmake curl pkgconf clang gtk3 fontconfig dav1d \
        libvpx libyuv opus aom ffmpeg xdotool libxcb libxfixes alsa-lib libpulse \
        gst-plugins-base-libs pipewire
```

### Build

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
git clone --recurse-submodules https://github.com/rustdesk/rustdesk
cd rustdesk
cargo run --locked
```

## How to build with Docker

Begin by cloning the repository and building the Docker container:

```sh
git clone https://github.com/rustdesk/rustdesk
cd rustdesk
git submodule update --init --recursive
docker build -t "rustdesk-builder" .
```

Then, each time you need to build the application, run the following command:

```sh
docker run --rm -it -v $PWD:/home/user/rustdesk -v rustdesk-git-cache:/home/user/.cargo/git -v rustdesk-registry-cache:/home/user/.cargo/registry -e PUID="$(id -u)" -e PGID="$(id -g)" rustdesk-builder
```

Note that the first build may take longer before dependencies are cached, subsequent builds will be faster. Additionally, if you need to specify different arguments to the build command, you may do so at the end of the command in the `<OPTIONAL-ARGS>` position. For instance, if you wanted to build an optimized release version, you would run the command above followed by `--release`. The resulting executable will be available in the target folder on your system, and can be run with:

```sh
target/debug/rustdesk
```

Or, if you're running a release executable:

```sh
target/release/rustdesk
```

Please ensure that you run these commands from the root of the RustDesk repository, or the application may not find the required resources. Also note that other cargo subcommands such as `install` or `run` are not currently supported via this method as they would install or run the program inside the container instead of the host.

## File Structure

- **[libs/hbb_common](https://github.com/rustdesk/hbb_common)**: video codec, config, tcp/udp wrapper, and some other utility functions shared with the server
- **[libs/base](https://github.com/rustdesk/rustdesk/tree/master/libs/base)**: protobuf, fs functions for file transfer, keyboard and platform code used only by this app
- **[libs/scrap](https://github.com/rustdesk/rustdesk/tree/master/libs/scrap)**: screen capture
- **[libs/enigo](https://github.com/rustdesk/rustdesk/tree/master/libs/enigo)**: platform specific keyboard/mouse control
- **[libs/clipboard](https://github.com/rustdesk/rustdesk/tree/master/libs/clipboard)**: file copy and paste implementation for Windows, Linux, macOS.
- **[src/dioxus_ui/](src/dioxus_ui/)**: native desktop and Android interface
- **[src/server](https://github.com/rustdesk/rustdesk/tree/master/src/server)**: audio/clipboard/input/video services, and network connections
- **[src/client.rs](https://github.com/rustdesk/rustdesk/tree/master/src/client.rs)**: start a peer connection
- **[src/rendezvous_mediator.rs](https://github.com/rustdesk/rustdesk/tree/master/src/rendezvous_mediator.rs)**: Communicate with [rustdesk-server](https://github.com/rustdesk/rustdesk-server), wait for remote direct (TCP hole punching) or relayed connection
- **[src/platform](https://github.com/rustdesk/rustdesk/tree/master/src/platform)**: platform specific code

## Screenshots

![Connection Manager](https://github.com/rustdesk/rustdesk/assets/28412477/db82d4e7-c4bc-4823-8e6f-6af7eadf7651)

![Connected to a Windows PC](https://github.com/rustdesk/rustdesk/assets/28412477/9baa91e9-3362-4d06-aa1a-7518edcbd7ea)

![File Transfer](https://github.com/rustdesk/rustdesk/assets/28412477/39511ad3-aa9a-4f8c-8947-1cce286a46ad)

![TCP Tunneling](https://github.com/rustdesk/rustdesk/assets/28412477/78e8708f-e87e-4570-8373-1360033ea6c5)

