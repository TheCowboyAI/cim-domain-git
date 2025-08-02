# Graph Analysis Features

## Overview

The cim-domain-git crate now includes powerful analyzers for extracting Git metadata optimized for building graphs with cim-ipld and cim-domain-graphs. This enables visualization and analysis of:

- **Social collaboration networks** - Who works with whom
- **Code ownership patterns** - Primary contributors to different parts of the codebase
- **Team clusters** - Natural team formations based on collaboration
- **Technical debt heatmaps** - Areas of the codebase needing attention
- **Repository health metrics** - Overall project health indicators

## Analyzers

### CollaborationAnalyzer

Detects collaboration patterns between developers:

```rust
use cim_domain_git::analyzers::CollaborationAnalyzer;

let analyzer = CollaborationAnalyzer::new();

// Analyze collaboration patterns
let collaborations = analyzer.analyze_collaboration(
    repository_id,
    &commits, // Vec of (hash, author, files, timestamp)
);

// Calculate code ownership
let ownership = analyzer.calculate_ownership(
    repository_id,
    &file_commits, // HashMap<FilePath, Vec<(Author, Timestamp)>>
);

// Detect team clusters
let teams = analyzer.detect_team_clusters(
    repository_id,
    &collaborations,
    min_team_size,
);
```

### CodeQualityAnalyzer

Analyzes code quality metrics for technical debt visualization:

```rust
use cim_domain_git::analyzers::CodeQualityAnalyzer;

let analyzer = CodeQualityAnalyzer::new();

// Analyze file complexity
let complexity = analyzer.analyze_file_complexity(
    repository_id,
    file_path,
    metrics,
);

// Calculate file churn (change frequency)
let churn = analyzer.calculate_file_churn(
    repository_id,
    file_path,
    &commits,
    Some(&metrics),
);

// Identify technical debt hotspots
let debt = analyzer.identify_technical_debt(
    repository_id,
    file_analysis,
);

// Calculate repository health
let health = analyzer.calculate_repository_health(
    repository_id,
    active_contributors,
    commits_last_week,
    total_branches,
    stale_branches,
    critical_issues,
);
```

## Events

The analyzers emit various events that can be streamed to graph databases:

### Collaboration Events

- `CollaborationDetected` - When developers work on the same files
- `CodeOwnershipCalculated` - Primary contributors to files/directories  
- `TeamClusterDetected` - Natural team formations
- `CommitRelationshipDetected` - Cherry-picks, reverts, fixes
- `CodeReviewDetected` - Review patterns between developers

### Code Quality Events

- `FileComplexityAnalyzed` - Complexity metrics for files
- `FileChurnCalculated` - Change frequency and risk level
- `TechnicalDebtIdentified` - Various types of technical debt
- `RepositoryHealthCalculated` - Overall health metrics
- `CircularDependencyDetected` - Dependency cycles

## Integration Example

```rust
// Extract commits from Git repository
let commits = extract_commits_from_repo(&repo_path)?;

// Analyze collaborations
let collab_analyzer = CollaborationAnalyzer::new();
let collaborations = collab_analyzer.analyze_collaboration(repo_id, &commits);

// Stream events to graph database
for collab in collaborations {
    // Convert to graph nodes and edges
    let author1_node = create_author_node(&collab.authors[0]);
    let author2_node = create_author_node(&collab.authors[1]);
    let collab_edge = create_collaboration_edge(
        &author1_node, 
        &author2_node,
        collab.collaboration_strength,
    );
    
    // Store in graph database
    graph_db.add_nodes(&[author1_node, author2_node])?;
    graph_db.add_edge(collab_edge)?;
}
```

## Use Cases

1. **Developer Productivity Analysis**
   - Identify key contributors
   - Find collaboration bottlenecks
   - Optimize team composition

2. **Code Quality Monitoring**
   - Track technical debt accumulation
   - Identify high-risk areas
   - Plan refactoring efforts

3. **Team Dynamics Visualization**
   - Understand team structures
   - Improve collaboration
   - Onboard new developers

4. **Project Health Dashboards**
   - Monitor repository health
   - Track contributor activity
   - Identify stale code

## Running the Example

See the full example in `examples/graph_analysis_demo.rs`:

```bash
cargo run --example graph_analysis_demo
```

This demonstrates all the analyzers and shows the types of metadata extracted for graph building.