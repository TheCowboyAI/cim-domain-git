# NixOS Development Standards

## Core Understanding

- NixOS is a Linux distribution with a unique declarative approach to system configuration
- System state is determined entirely by configuration files (primarily in Nix language)
- System changes require rebuilding with `nixos-rebuild` (never modify system files directly)
  - **THIS MUST BE DONE BY A HUMAN**
- There is ALWAYS a .direnv, a local development shell for NixOS
  - **YOUR SHELL WILL ALWAYS EXECUTE THERE**
- Package management is handled through the Nix store with immutable, atomic operations
- System configurations are designed to be reproducible and portable
- **NixOS REQUIRES files to be staged to be seen, STAGE NEW FILES WHEN BUILDING**

## Module Structure

Each NixOS module handles one logical aspect of the configuration:

```nix
{ config, pkgs, ... }:

{
  imports = [
    # Paths of other modules
  ];
  
  options = {
    # Option declarations
  };
  
  config = {
    # Option definitions
  };
}
```

## Configuration Best Practices

- Use git for version control of NixOS configurations
- Apply configuration changes with `nixos-rebuild switch --flake .#hostname`
  - **MUST BE DONE BY A HUMAN**
- Debug issues with `--show-trace --print-build-logs --verbose` flags
- Store personal configurations in `~/nixos-config` with symlink from `/etc/nixos`
- Prefer declarative configuration over imperative changes
- Create modular configurations with clear separation of concerns
- Always make a devshell available via direnv

## Nix Language Guidelines

- Leverage functional programming paradigms (immutability, pure functions)
- Use `let ... in` expressions for local variables
- Avoid `with` expressions as they can cause namespace pollution
- Format Nix code consistently using `nixpkgs-fmt` or `alejandra`
- Comment complex expressions for better maintainability
- Use string interpolation with care (`"${...}"`)
- **NEVER USE `heredoc` SYNTAX IN NIX FILES, FORMATTERS BREAK THEM**
- **IF YOU ENCOUNTER EOF ON A LINE BY ITSELF IN A NIX FILE, BE SURE IT HAS ZERO WHITESPACE ON THE LINE BEFORE `EOF`**

## Flakes Support

- Structure projects with `flake.nix` in the repository root
- Define clear inputs with pinned versions
- Specify nixosConfigurations output for system configurations
- Use `specialArgs` to pass additional parameters to modules
- Track flake.lock in version control for reproducibility
- Update dependencies with `nix flake update`

### Example Flake Structure
```nix
{
  description = "CIM module flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
          ];
        };
      });
}
```

## Package Management

- Define system packages in `environment.systemPackages`
- Create development environments using `flake.nix` and `devShells` output
- Override existing packages with `pkgs.override` or `pkgs.overrideAttrs`
- Use overlays for consistent package modifications
- Leverage `nixpkgs.config` options for global package configuration
- Install user packages through home-manager when appropriate

## Service Configuration

- Define services using appropriate `services.*` options
- Create custom services with `systemd.services.<name>`
- Set proper dependencies with `after`, `wants`, `requires`, etc.
- Configure networking with `networking.*` options
- Handle user accounts with `users.users.<name>` options
- Use `systemd.tmpfiles.rules` for filesystem setup

### Example Service
```nix
systemd.services.cim-leaf = {
  description = "CIM Leaf Node Service";
  after = [ "network.target" "nats.service" ];
  wantedBy = [ "multi-user.target" ];
  
  serviceConfig = {
    Type = "notify";
    ExecStart = "${pkgs.cim-leaf}/bin/cim-leaf";
    Restart = "on-failure";
    RestartSec = "5s";
  };
};
```

## Cross-Platform Considerations

- Use `lib.mkIf` for conditional configurations
- Check system type with `pkgs.stdenv.hostPlatform.system`
- Structure configurations to support multiple machines
- Leverage hardware-detection modules for portable configurations
- Create abstraction layers for hardware-specific requirements

## Error Handling and Debugging

- Set `warnings` or `assertions` for runtime validation
- Monitor service issues with `journalctl -u service-name`
- Check build logs in `/var/log/nixos`
- Use `nix-store --verify` to check for store corruption
- Execute `nix why-depends` to understand package dependencies
- Run `nix-store --gc` to clean up unused packages

## CIM-Specific NixOS Patterns

### Module Development
```nix
# cim-module.nix
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.cim-module;
in {
  options.services.cim-module = {
    enable = mkEnableOption "CIM module service";
    
    package = mkOption {
      type = types.package;
      default = pkgs.cim-module;
      description = "CIM module package to use";
    };
    
    natsUrl = mkOption {
      type = types.str;
      default = "nats://localhost:4222";
      description = "NATS server URL";
    };
  };
  
  config = mkIf cfg.enable {
    systemd.services.cim-module = {
      # Service configuration
    };
  };
}
```

### Development Shell
```nix
# shell.nix or devShell in flake.nix
pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rust-analyzer
    clippy
    rustfmt
    
    # NATS tools
    nats-server
    natscli
    
    # Development tools
    git
    direnv
    gnumake
  ];
  
  shellHook = ''
    echo "CIM Development Environment"
    echo "Run 'cargo test' to run tests"
    echo "Run 'nats-server' to start local NATS"
  '';
}
```

## Best Practices Summary

1. **Always use declarative configuration**
2. **Never modify system files directly**
3. **Stage new files in git before building**
4. **Use flakes for reproducibility**
5. **Test configurations in VMs before deploying**
6. **Keep modules focused and composable**
7. **Document all custom options**
8. **Use type checking for option validation**