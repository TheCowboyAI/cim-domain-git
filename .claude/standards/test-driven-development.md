# Test-Driven Development Standards

## Core TDD Principles

1. **Test-First Development** - Never write production code without a failing test first
2. **Domain Isolation** - Domain logic tests MUST NOT contain Bevy/NATS dependencies
3. **Headless Execution** - All tests must run with `BEVY_HEADLESS=1` mode
4. **Test Coverage** - Maintain 95%+ coverage

## Event and Command Handling

### Event Handlers
- ALL Events must have a handler (even if not always called)
- Every handler requires tests
- Include Mermaid graphs in rustdocs showing test flow

### Command Handlers
- ALL Commands must have a handler (even if not always called)
- Every handler requires tests
- Document command flow with Mermaid diagrams

### Documentation Requirement
```rust
/// Tests the order processing command handler
/// 
/// ```mermaid
/// graph LR
///     Command[ProcessOrder] --> Handler[OrderHandler]
///     Handler --> Event[OrderProcessed]
///     Event --> Result[Updated State]
/// ```
#[test]
fn test_process_order_command() {
    // Test implementation
}
```

## Testing Patterns

### Bevy ECS Testing
```rust
#[test]
fn test_ecs_system() {
    let mut app = App::new();
    app.add_systems(Update, system_under_test)
       .insert_resource(TestNatsClient::new());

    app.update();
    
    let results = app.world.query::<&Component>().iter(&app.world);
    assert_eq!(results.len(), 1);
}
```

### NATS Message Validation
```rust
#[test]
fn validate_nats_message_handling() {
    let mut app = App::new();
    app.add_systems(Update, nats_bridge_system);
    
    // Inject test message
    app.world.send_event(NatsIncoming {
        subject: "test.subject".to_string(),
        payload: json!({"test": "data"}),
    });
    
    app.update();
    
    // Verify response
    let events = app.world.resource::<Events<NatsOutgoing>>();
    assert_eq!(events.len(), 1);
}
```

### Domain Service Testing
```rust
#[test]
fn test_domain_service() {
    // Pure domain logic - no ECS/NATS dependencies
    let input = DomainInput { /* ... */ };
    let result = DomainService::process(input);
    
    assert!(result.is_ok());
    match result {
        Ok(output) => assert_eq!(output.status, Status::Processed),
        Err(_) => panic!("Processing should succeed"),
    }
}
```

## Environment Configuration

### Required Flake Structure
```nix
{
  inputs.bevy.url = "github:bevyengine/bevy/v0.16";

  outputs = { self, nixpkgs, bevy }:
    let pkgs = nixpkgs.legacyPackages.x86_64-linux;
    in {
      devShells.default = pkgs.mkShell {
        BEVY_HEADLESS = "1";
        buildInputs = with pkgs; [
          rustc cargo pkg-config
          vulkan-headers wayland
          nats-server natscli
        ];
      };
    };
}
```

## TDD Workflow

### Setup Commands
```bash
# Start TDD session
direnv allow && nix develop

# Run tests in watch mode
BEVY_HEADLESS=1 cargo watch -x test

# NATS integration tests
nats-server -js & cargo test --test nats_bridge

# Domain layer tests only
cargo test --lib -- domain
```

## Prohibited Patterns

❌ **Never use** `#[cfg(test)]` without `#[test]`
❌ **No direct** Wayland dependencies - use NixOS graphics stack
❌ **Ban** `unwrap()` in domain logic - handle Option/Result properly
❌ **Avoid** rendering tests - test that rendering data exists instead

## Test Structure Requirements

### Test Module Organization
- Test modules MUST mirror src structure
- Integration tests in `tests/` directory
- Unit tests in same file as code

### Test Naming
```rust
#[test]
fn test_component_creation() { }

#[test]
fn test_system_processes_events() { }

#[test]
fn test_handler_emits_correct_events() { }
```

## Performance Requirements

| Test Type | Max Duration | Memory Limit |
|-----------|--------------|--------------|
| Unit Test | <100ms | <50MB |
| Integration Test | <500ms | <100MB |
| System Test | <1000ms | <200MB |

### Performance Guidelines
1. Single test duration must be <100ms
2. Test memory usage must be <50MB
3. No async in domain layer tests

## Code Generation Rules

### ECS Components
```rust
#[derive(Component, Clone, Debug)]
struct MyComponent {
    // fields
}
```

### NATS Messages
```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
struct MyMessage {
    pub id: String,
    pub payload: JsonValue,
}
```

## Error Handling in Tests

### Test Failure Pattern
```rust
#[test]
#[should_panic(expected = "specific error message")]
fn test_critical_failure() {
    // Code that should panic
}
```

### Assertion Patterns
```rust
// Use descriptive assertions
assert_eq!(actual, expected, "Component should have initial value");

// Check multiple conditions
assert!(
    result.is_ok() && result.unwrap().len() > 0,
    "Result should be Ok with non-empty data"
);
```

## Verification Matrix

| Test Type | Execution | Must Pass | Coverage Target |
|-----------|-----------|-----------|-----------------|
| Unit (Domain) | Every commit | Yes | 100% |
| Integration (ECS) | Every commit | Yes | 95% |
| NATS Bridge | Before merge | Yes | 90% |
| End-to-End | Before release | Yes | 85% |

## Test Quality Standards

1. **Clear Test Names** - Describe what is being tested
2. **Single Assertion Focus** - One logical assertion per test
3. **Isolated Tests** - No dependencies between tests
4. **Deterministic** - Same result every time
5. **Fast Execution** - Optimize for speed
6. **Meaningful Failures** - Clear error messages