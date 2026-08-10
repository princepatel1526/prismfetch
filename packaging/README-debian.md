# Building a .deb with cargo-deb (Debian/Ubuntu-based distros)

1. Install the tool once:
   cargo install cargo-deb

2. From the project root, add this to Cargo.toml (already included below the
   [dependencies] section in this project — see the [package.metadata.deb]
   block) and then run:
   cargo deb

3. The finished package appears at:
   target/debian/prismfetch_1.0.0-1_amd64.deb

4. Install it with:
   sudo dpkg -i target/debian/prismfetch_1.0.0-1_amd64.deb
