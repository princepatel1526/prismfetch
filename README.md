# prismfetch

A `neofetch`-style system-info tool for **PrismOS**, written in Rust. Prints
a colored block-grid logo next to key system stats, plus gradient usage bars
for CPU, memory, disk, and temperature — matching the PrismOS Aurora theme.

## 1. Install prerequisites

You need the Rust toolchain and a C linker. On a Debian/Ubuntu-based base:

```bash
sudo apt update
sudo apt install -y curl build-essential pkg-config
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

On an Arch-based base:

```bash
sudo pacman -S --needed rustup base-devel
rustup default stable
```

Verify:

```bash
rustc --version
cargo --version
```

## 2. Project layout

```
prismfetch/
├── Cargo.toml            # dependencies + package metadata
├── src/
│   ├── main.rs            # entry point
│   ├── ascii.rs            # the PrismOS block-grid logo
│   ├── collector.rs        # gathers all system data
│   └── render.rs            # prints the logo, fields, and bars
├── install.sh              # builds + installs to /usr/local/bin
└── packaging/
    ├── PKGBUILD              # for pacman-based PrismOS builds
    └── README-debian.md       # for cargo-deb (.deb) builds
```

## 3. Dependencies (`Cargo.toml`)

PrismFetch has **no runtime or third-party Rust dependencies**. It reads Linux
`/proc` and `/sys` interfaces directly, with small libc calls for terminal and
disk size. `lspci` is optional and improves GPU identification; `pacman` or
`dpkg-query` is optional and supplies the package count.

## 4. Build

```bash
cd prismfetch
cargo build --release
```

The binary is produced at `target/release/prismfetch`.

Run it directly to test:

```bash
./target/release/prismfetch
```

## 5. Install system-wide

Quick way, using the provided script:

```bash
./install.sh
```

This puts the binary at `/usr/local/bin/prismfetch`, so any user can just
type `prismfetch`.

## 6. Package it properly for PrismOS

Pick whichever matches your base distro:

### If PrismOS is Arch-based (pacman)
Use `packaging/PKGBUILD`. Put the source in a tarball named
`prismfetch-1.0.0.tar.gz` next to it, then:

```bash
cd packaging
makepkg -si
```

### If PrismOS is Debian-based (dpkg/apt)
See `packaging/README-debian.md` — uses `cargo-deb`, and the
`[package.metadata.deb]` block already in `Cargo.toml` describes the package.

```bash
cargo install cargo-deb
cargo deb
sudo dpkg -i target/debian/prismfetch_1.0.0-1_amd64.deb
```

## 7. Run on every terminal launch (optional)

To have it show automatically, like the screenshot, add this line to your
shell's rc file (e.g. `/etc/skel/.bashrc` or `/etc/skel/.zshrc` so new users
get it too):

```bash
prismfetch
```

## 8. Customizing the look

- **Logo** — edit the `LOGO` grid and the `BLUE` / `MAGENTA` / `PURPLE`
  `TrueColor` values in `src/ascii.rs`.
- **Fields shown** — edit the `rows` array in `src/render.rs`.
- **Bar colors / width** — edit the palette and `meter()` in `src/render.rs`.
- **OS name** — `collector.rs` reads `PRETTY_NAME` from `/etc/os-release`,
  so set that file correctly when you build your PrismOS root filesystem,
  e.g.:
  ```
  NAME="PrismOS"
  PRETTY_NAME="PrismOS 1.0 Aurora"
  ID=prismos
  ```

## How each field is actually gathered

| Field | Source |
|---|---|
| OS | `/etc/os-release` → `PRETTY_NAME` |
| Kernel | `/proc/sys/kernel/osrelease` |
| Shell | `PRISM_SHELL_NAME`, else Prism Shell |
| Compositor / Window Manager | `$XDG_SESSION_TYPE` / `$XDG_CURRENT_DESKTOP` |
| Terminal | `PRISM_TERMINAL_NAME` / `$TERM_PROGRAM`, else Prism Terminal |
| CPU | `/proc/cpuinfo` |
| GPU | `lspci` output, first VGA/3D controller line |
| RAM | `/proc/meminfo` |
| Packages | `pacman -Qq`, then `dpkg-query`; otherwise N/A |
| Theme / Icons / Font | `PRISM_THEME`, `PRISM_ICON_THEME`, `PRISM_FONT`, else PrismOS defaults |
| Uptime | `/proc/uptime` |
| CPU/Mem/Disk/Temp bars | `/proc/loadavg`, `/proc/meminfo`, `statvfs`, `/sys` thermal sensors |

This utility is dependency-free and builds with the stable Rust toolchain — you should still run `cargo build --release` yourself on
your target machine before packaging, since GPU/theme detection depends on
what's actually installed there.


## Layout and terminal behavior

PrismFetch reads the terminal width from `$COLUMNS` or the Linux terminal ioctl.
At 100 columns and above it uses the side-by-side reference layout; below 100 it
uses a stacked layout that cannot wrap. Set `NO_COLOR=1`, use `TERM=dumb`, or
redirect stdout to disable ANSI styling. Linux virtual consoles use an ASCII
fallback for rules and meters; UTF-8 terminal emulators get block graphics and
24-bit Aurora colors.

PrismOS branding can be supplied by `/etc/os-release`; session branding can be
overridden with `PRISM_SHELL_NAME`, `PRISM_WM_NAME`, `PRISM_TERMINAL_NAME`,
`PRISM_THEME`, `PRISM_ICON_THEME`, and `PRISM_FONT`.
