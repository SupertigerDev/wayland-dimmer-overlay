# Wayland Dimmer Overlay

Software/virtual screen dimmer below 0% using rust. 

# Tested on
- KDE Plasma

Please test on more environments and report issues!

# Usage
Run the compiled binary with an optional brightness argument (0.0 to 1.0). Default is 0.3.

```bash
./wayland-dimmer-overlay 0.6
```

# Build Instructions
1. Install Rust and Cargo from https://rustup.rs/
2. Clone this repository using `git clone https://github.com/SupertigerDev/wayland-dimmer-overlay.git`
3. Navigate to the project directory: `cd wayland-dimmer-overlay`
4. Build the project using Cargo: `cargo build --release`
5. The compiled binary will be located in the `target/release` directory.

