# New CIM Creation Workflow

## Step-by-Step Process for Creating a Domain-Specific CIM

### Step 1: Clone cim-start Template
```bash
# Clone the starting template
git clone <cim-start-repository> cim-domain-<name>
cd cim-domain-<name>

# Update project metadata
sed -i 's/cim-start/cim-domain-<name>/g' Cargo.toml
sed -i 's/CIM Start/CIM <Domain Name>/g' README.md
```

### Step 2: Initialize Development Environment
```bash
# Enter the Nix development shell
direnv allow
nix develop

# Verify NATS is configured
nats-server --version

# Run initial tests
cargo test
```

### Step 3: Domain Analysis
Create a domain specification:
```yaml
# domain-spec.yaml
domain:
  name: "Private Mortgage Lending"
  code: "mortgage"
  
entities:
  - Loan
  - Borrower
  - Property
  - Underwriting
  
workflows:
  - loan_application
  - credit_check
  - property_appraisal
  - underwriting_decision
  - loan_closing
  
policies:
  - lending_regulations
  - risk_assessment
  - document_requirements
  
integrations:
  - credit_bureaus
  - property_databases
  - banking_systems
```

### Step 4: Select Required Modules
Based on domain analysis, add dependencies:
```toml
# Cargo.toml
[dependencies]
# Core modules (from cim-start)
cim-domain = { path = "../cim-domain" }
cim-infrastructure = { path = "../cim-infrastructure" }

# Selected modules for mortgage domain
cim-domain-identity = { path = "../cim-domain-identity" }
cim-domain-document = { path = "../cim-domain-document" }
cim-domain-workflow = { path = "../cim-domain-workflow" }
cim-domain-policy = { path = "../cim-domain-policy" }
cim-security = { path = "../cim-security" }
cim-flashstor = { path = "../cim-flashstor" }
```

### Step 5: Create Domain Module Structure
```bash
# Create domain-specific directories
mkdir -p src/domain
mkdir -p src/workflows
mkdir -p src/policies
mkdir -p src/integrations

# Create module files
touch src/domain/mod.rs
touch src/domain/loan.rs
touch src/domain/borrower.rs
touch src/workflows/mod.rs
touch src/workflows/loan_application.rs
touch src/policies/mod.rs
touch src/policies/lending_compliance.rs
```

### Step 6: Implement Domain Events
```rust
// src/domain/events.rs
use cim_domain::DomainEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MortgageDomainEvent {
    // Loan lifecycle events
    LoanApplicationSubmitted { ... },
    CreditCheckCompleted { ... },
    PropertyAppraised { ... },
    UnderwritingDecisionMade { ... },
    LoanClosed { ... },
    
    // Use existing events from other modules
    IdentityVerified(cim_domain_identity::IdentityVerified),
    DocumentUploaded(cim_domain_document::DocumentUploaded),
}
```

### Step 7: Configure Workflows
```rust
// src/workflows/loan_application.rs
use cim_domain_workflow::{WorkflowEngine, WorkflowDefinition};

pub fn configure_loan_workflows(engine: &mut WorkflowEngine) -> Result<()> {
    // Define the loan application workflow
    engine.register_workflow(
        "loan_application",
        loan_application_workflow(),
    )?;
    
    // Define other workflows
    engine.register_workflow(
        "underwriting",
        underwriting_workflow(),
    )?;
    
    Ok(())
}
```

### Step 8: Set Up NATS Subjects
```rust
// src/infrastructure/subjects.rs
pub const MORTGAGE_SUBJECTS: &[&str] = &[
    "mortgage.loan.application.submitted",
    "mortgage.loan.credit.checked",
    "mortgage.loan.property.appraised",
    "mortgage.loan.underwriting.completed",
    "mortgage.loan.closed",
];
```

### Step 9: Create Integration Tests
```rust
// tests/integration/loan_lifecycle.rs
#[tokio::test]
async fn test_complete_loan_lifecycle() {
    // Start with loan application
    // Verify each workflow step
    // Ensure events are properly published
    // Check final state
}
```

### Step 10: Update Progress and Documentation
```bash
# Update progress.json
CURRENT_DATE=$(date -I)
GIT_HASH=$(git rev-parse HEAD)

# Add domain creation to progress.json
# Document the new domain in README.md
# Create domain-specific documentation
```

## Checklist for New CIM

- [ ] Cloned cim-start template
- [ ] Updated project metadata
- [ ] Created domain specification
- [ ] Selected required modules
- [ ] Created domain module structure
- [ ] Defined domain events
- [ ] Configured workflows
- [ ] Set up NATS subjects
- [ ] Implemented core aggregates
- [ ] Created integration tests
- [ ] Updated documentation
- [ ] Updated progress.json

## Common Patterns

### Module Selection Guidelines
- **Always include**: cim-domain, cim-infrastructure
- **For user management**: cim-domain-identity
- **For documents**: cim-domain-document
- **For processes**: cim-domain-workflow
- **For rules**: cim-domain-policy
- **For storage**: cim-flashstor or cim-ipld

### Naming Conventions
- Repository: `cim-domain-<name>`
- Main module: `cim_domain_<name>`
- Events: `<Name>DomainEvent`
- Aggregates: `<Name>Aggregate`

### Testing Strategy
1. Unit tests for domain logic
2. Integration tests for workflows
3. NATS messaging tests
4. End-to-end scenario tests