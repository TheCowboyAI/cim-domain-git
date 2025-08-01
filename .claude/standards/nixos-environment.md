# NixOS Environment Standards

## Core Understanding

- **Declarative System**: State determined by configuration files
- **Immutable Operations**: Changes via `nixos-rebuild` only (MUST BE DONE BY HUMAN)
- **Nix Store**: Package management with atomic operations
- **Reproducible**: Configurations are portable and reproducible
- **Local Dev Shell**: Always use .direnv for development
- **File Staging**: New files must be staged to be seen by NixOS

## Module Structure

Basic NixOS module structure:

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

### Module Best Practices

1. **Single Responsibility**: Each module handles one logical aspect
2. **Option Declaration**: Modules can declare options for others to define
3. **Option Definition**: Modules can define options declared by others
4. **Clear Separation**: Maintain separation of concerns

## Configuration Management

### Version Control
- Use git for NixOS configurations
- Store in `~/nixos-config` with symlink from `/etc/nixos`

### Applying Changes
```bash
# Apply configuration (HUMAN ACTION REQUIRED)
nixos-rebuild switch --flake .#hostname

# Debug with verbose output
nixos-rebuild switch --show-trace --print-build-logs --verbose
```

### Best Practices
1. Prefer declarative over imperative changes
2. Create modular configurations
3. Always provide devshell via direnv
4. Use the NixOS MCP tool to clarify package options

## Nix Language Guidelines

### Do's
- Use functional programming paradigms
- Use `let ... in` for local variables
- Format with `nixpkgs-fmt` or `alejandra`
- Comment complex expressions
- Use string interpolation carefully: `"${...}"`

### Don'ts
- Avoid `with` expressions (namespace pollution)
- NEVER use heredoc syntax (formatters break them)
- Ensure EOF lines have ZERO whitespace

## Flakes Support

### Structure
```nix
{
  description = "System configuration";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Other inputs with pinned versions
  };
  
  outputs = { self, nixpkgs, ... }: {
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      specialArgs = { inherit inputs; };
      modules = [ ./configuration.nix ];
    };
  };
}
```

### Management
- Track `flake.lock` in version control
- Update with `nix flake update`
- Use `specialArgs` for module parameters

## Package Management

### System Packages
```nix
environment.systemPackages = with pkgs; [
  git
  vim
  # Other packages
];
```

### Development Environments
- Use `flake.nix` with `devShells` output
- Override packages with `pkgs.override` or `pkgs.overrideAttrs`
- Use overlays for consistent modifications
- Install user packages via home-manager when appropriate

## Service Configuration

### System Services
```nix
# Using existing service modules
services.nginx.enable = true;

# Custom service
systemd.services.myservice = {
  description = "My custom service";
  after = [ "network.target" ];
  wantedBy = [ "multi-user.target" ];
  serviceConfig = {
    ExecStart = "${pkgs.myapp}/bin/myapp";
    Restart = "always";
  };
};
```

### Configuration Elements
- Define services with `services.*` options
- Create custom services with `systemd.services`
- Set dependencies: `after`, `wants`, `requires`
- Configure networking: `networking.*`
- Manage users: `users.users.*`
- Filesystem setup: `systemd.tmpfiles.rules`

## Cross-Platform Support

### Conditional Configuration
```nix
{
  config = lib.mkIf (pkgs.stdenv.hostPlatform.system == "x86_64-linux") {
    # Linux-specific configuration
  };
}
```

### Portability
- Check system type before applying platform-specific config
- Structure for multiple machines support
- Use hardware-detection modules
- Create abstraction layers for hardware differences

## Error Handling and Debugging

### Runtime Validation
```nix
{
  warnings = [ "This configuration requires X" ];
  assertions = [{
    assertion = config.services.nginx.enable -> config.networking.firewall.enable;
    message = "Nginx requires firewall to be enabled";
  }];
}
```

### Debugging Commands
```bash
# Monitor service logs
journalctl -u service-name

# Check build logs
ls /var/log/nixos

# Verify store integrity
nix-store --verify

# Understand dependencies
nix why-depends package1 package2

# Clean unused packages
nix-store --gc
```

## Best Practices Summary

1. **Modularity**: Keep configurations modular and focused
2. **Reproducibility**: Pin versions and track lock files
3. **Documentation**: Comment complex configurations
4. **Testing**: Test changes in VM before applying to system
5. **Backups**: Keep configuration backups before major changes
6. **Gradual Changes**: Apply changes incrementally
7. **Tool Usage**: Use NixOS MCP tool for package exploration