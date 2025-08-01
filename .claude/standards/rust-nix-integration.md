# Rust with Nix Integration Standards

## Overview

Standards for Rust development in NixOS environments, specifically for Bevy/Wayland applications using Nightly Rust.

## Core Principles

1. **Never downgrade Rust libraries** - Only upgrade versions specified in Cargo.toml
2. **Use Nix build environment** - Always use `nix build` or `nix develop -c cargo build`
3. **Respect documentation** - Follow /doc guidelines before writing code
4. **No orphaned code** - Implement all functions and use all variables created
5. **Stage new files** - NixOS requires files to be staged to be seen

## Development Environment Setup

### Toolchain Management

Use oxalica/rust-overlay with flake-parts:

```nix
rust-nightly = (pkgs.rust-bin.selectLatestNightlyWith (toolchain:
  toolchain.default.override {
    extensions = ["rust-src" "rust-analyzer"];
    targets = ["x86_64-unknown-linux-gnu"];
  }
));
```

### Shell Environment

```nix
shellHook = ''
  export RUST_SRC_PATH="${rust-nightly}/lib/rustlib/src/rust/library"
  export WINIT_UNIX_BACKEND=wayland
  export RUST_BACKTRACE=full
'';
```

### Required Build Inputs

```nix
buildInputs = with pkgs; [
  vulkan-loader
  libxkbcommon
  wayland
  udev
  alsaLib
  pkg-config
  xorg.libX11
];
```

## Bevy Dynamic Linking Configuration

### Development Features

```toml
[features]
dev = [
  "bevy/dynamic_linking",
  "bevy/asset_processor",
  "bevy/file_watcher"
]
```

### Development Shell

```nix
devShells.default = pkgs.mkShell {
  packages = [rust-nightly];
  buildInputs = with pkgs; [
    vulkan-loader
    libxkbcommon
    wayland
    udev
  ];

  shellHook = ''
    export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
    export CARGO_FEATURES_DEV="--features dev"
  '';
};
```

## Production Build Configuration

### Nix Package Derivation

```nix
packages.default = pkgs.rustPlatform.buildRustPackage {
  cargoLock.lockFile = ./Cargo.lock;

  buildInputs = with pkgs; [
    vulkan-loader
    wayland
  ];

  cargoBuildFlags = "--release --no-default-features";

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [
    pkgs.vulkan-loader
    pkgs.wayland
  ]}";
};
```

## Build Commands

| Environment | Command | Flags |
|-------------|---------|-------|
| Development | `nix develop` | (shellHook auto-applies) |
| Production | `nix build -L --impure` | `--option sandbox relaxed` |

### Critical Build Arguments

```bash
# For systems with proprietary drivers
nix build --impure --option binary-caches "https://cache.nixos.org https://nixpkgs-wayland.cachix.org"
```

## Dependency Matrix

| Library | Development | Production |
|---------|-------------|------------|
| vulkan-loader | ✓ Dynamic | ✓ Shared |
| libxkbcommon | ✓ Dynamic | ✓ Shared |
| wayland-protocols | ✓ Dynamic | ✗ Static |

## Project Structure

Follow Rust conventions:
- `src/` - Source code
- `main.rs`/`lib.rs` - Entry points
- `bin/` - Multiple binaries
- `tests/` - Integration tests
- `examples/` - Example code

## Validation and Testing

### Pre-commit Checks

```bash
# Check for accidental dynamic linking in release builds
! nix eval .#packages.${system}.default | grep "bevy_dylib"
```

### CI Pipeline Check

```yaml
- name: Verify static linking
  run: |
    readelf -d result/bin/bevy_app | grep -q 'NOTYPE.*GLOBAL DEFAULT.*bevy_dylib' && \
      (echo "Dynamic linking detected!"; exit 1)
```

## Common Issues and Solutions

### Missing Vulkan Layers
Add to Nix inputs:
```nix
vulkan-validation-layers
glslang
```

### Wayland Surface Creation Failures
Set in derivation:
```nix
XDG_RUNTIME_DIR = "/tmp";
```

### Shader Compilation Errors
Include in build inputs:
```nix
shaderc.override { preferVulkan = true; }
```

## Testing Commands

```bash
# Ensure no errors before building
nix develop -c cargo check
nix develop -c cargo test

# Build and test
nix develop -c cargo build
nix develop -c cargo test
```

## Best Practices

1. **Pin Nightly Rust version** for reproducibility
2. **Document setup** in README.md
3. **Use workspace** for multi-crate projects
4. **Follow examples** in /samples for Bevy and Egui syntax
5. **Test thoroughly** before building (builds take time)