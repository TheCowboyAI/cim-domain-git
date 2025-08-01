<!-- Copyright 2025 Cowboy AI, LLC. -->

# Git API Documentation

## Overview

The Git domain API provides commands, queries, and events for {domain purpose}.

## Commands

### CreateGit

Creates a new git in the system.

```rust
use cim_domain_git::commands::CreateGit;

let command = CreateGit {
    id: GitId::new(),
    // ... fields
};
```

**Fields:**
- `id`: Unique identifier for the git
- `field1`: Description
- `field2`: Description

**Validation:**
- Field1 must be non-empty
- Field2 must be valid

**Events Emitted:**
- `GitCreated`

### UpdateGit

Updates an existing git.

```rust
use cim_domain_git::commands::UpdateGit;

let command = UpdateGit {
    id: entity_id,
    // ... fields to update
};
```

**Fields:**
- `id`: Identifier of the git to update
- `field1`: New value (optional)

**Events Emitted:**
- `GitUpdated`

## Queries

### GetGitById

Retrieves a git by its identifier.

```rust
use cim_domain_git::queries::GetGitById;

let query = GetGitById {
    id: entity_id,
};
```

**Returns:** `Option<GitView>`

### List{Entities}

Lists all {entities} with optional filtering.

```rust
use cim_domain_git::queries::List{Entities};

let query = List{Entities} {
    filter: Some(Filter {
        // ... filter criteria
    }),
    pagination: Some(Pagination {
        page: 1,
        per_page: 20,
    }),
};
```

**Returns:** `Vec<GitView>`

## Events

### GitCreated

Emitted when a new git is created.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCreated {
    pub id: GitId,
    pub timestamp: SystemTime,
    // ... other fields
}
```

### GitUpdated

Emitted when a git is updated.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitUpdated {
    pub id: GitId,
    pub changes: Vec<FieldChange>,
    pub timestamp: SystemTime,
}
```

## Value Objects

### GitId

Unique identifier for {entities}.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GitId(Uuid);

impl GitId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
```

### {ValueObject}

Represents {description}.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct {ValueObject} {
    pub field1: String,
    pub field2: i32,
}
```

## Error Handling

The domain uses the following error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("git not found: {id}")]
    NotFound { id: GitId },
    
    #[error("Invalid {field}: {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("Operation not allowed: {reason}")]
    Forbidden { reason: String },
}
```

## Usage Examples

### Creating a New Git

```rust
use cim_domain_git::{
    commands::CreateGit,
    handlers::handle_create_git,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = CreateGit {
        id: GitId::new(),
        name: "Example".to_string(),
        // ... other fields
    };
    
    let events = handle_create_git(command).await?;
    
    for event in events {
        println!("Event emitted: {:?}", event);
    }
    
    Ok(())
}
```

### Querying {Entities}

```rust
use cim_domain_git::{
    queries::{List{Entities}, execute_query},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = List{Entities} {
        filter: None,
        pagination: Some(Pagination {
            page: 1,
            per_page: 10,
        }),
    };
    
    let results = execute_query(query).await?;
    
    for item in results {
        println!("{:?}", item);
    }
    
    Ok(())
}
```

## Integration with Other Domains

This domain integrates with:

- **{Other Domain}**: Description of integration
- **{Other Domain}**: Description of integration

## Performance Considerations

- Commands are processed asynchronously
- Queries use indexed projections for fast retrieval
- Events are published to NATS for distribution

## Security Considerations

- All commands require authentication
- Authorization is enforced at the aggregate level
- Sensitive data is encrypted in events 