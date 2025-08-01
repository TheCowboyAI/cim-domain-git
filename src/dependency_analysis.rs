// Copyright 2025 Cowboy AI, LLC.

//! Dependency analysis for Git repositories
//!
//! This module provides functionality to analyze files and extract
//! dependency information based on programming language.

use crate::GitDomainError;
use std::collections::{HashMap, HashSet};
use regex::Regex;

/// Represents a dependency found in a file
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    /// The name of the dependency
    pub name: String,
    /// The type of dependency (e.g., "import", "require", "use")
    pub dependency_type: DependencyType,
    /// Optional version specification
    pub version: Option<String>,
}

/// Types of dependencies
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyType {
    /// Language import (e.g., Python import, Java import)
    Import,
    /// Package dependency (e.g., npm, cargo, pip)
    Package,
    /// Include directive (e.g., C/C++ #include)
    Include,
    /// Module use (e.g., Rust use)
    Use,
    /// Other dependency type
    Other(String),
}

/// Programming language
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    /// Rust programming language
    Rust,
    /// Python programming language
    Python,
    /// JavaScript programming language
    JavaScript,
    /// TypeScript programming language
    TypeScript,
    /// Java programming language
    Java,
    /// Go programming language
    Go,
    /// C programming language
    C,
    /// C++ programming language
    Cpp,
    /// Other programming language
    Other(String),
}

impl Language {
    /// Detect language from file extension
    #[must_use] pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "js" | "mjs" | "cjs" => Language::JavaScript,
            "ts" | "tsx" => Language::TypeScript,
            "java" => Language::Java,
            "go" => Language::Go,
            "c" | "h" => Language::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Language::Cpp,
            other => Language::Other(other.to_string()),
        }
    }
}

/// Analyzes file content to extract dependencies
pub struct DependencyAnalyzer {
    /// Language-specific regex patterns
    patterns: HashMap<Language, Vec<(Regex, DependencyType)>>,
}

impl DependencyAnalyzer {
    /// Create a new dependency analyzer
    #[must_use] pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Rust patterns
        patterns.insert(Language::Rust, vec![
            (Regex::new(r"^\s*use\s+([a-zA-Z0-9_:]+)").unwrap(), DependencyType::Use),
            (Regex::new(r"^\s*extern\s+crate\s+([a-zA-Z0-9_]+)").unwrap(), DependencyType::Import),
        ]);
        
        // Python patterns
        patterns.insert(Language::Python, vec![
            (Regex::new(r"^\s*import\s+([a-zA-Z0-9_.]+)").unwrap(), DependencyType::Import),
            (Regex::new(r"^\s*from\s+([a-zA-Z0-9_.]+)\s+import").unwrap(), DependencyType::Import),
        ]);
        
        // JavaScript/TypeScript patterns
        let js_patterns = vec![
            (Regex::new(r#"^\s*import\s+.*\s+from\s+['"]([^'"]+)['"]"#).unwrap(), DependencyType::Import),
            (Regex::new(r#"^\s*const\s+.*\s*=\s*require\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap(), DependencyType::Import),
            (Regex::new(r#"^\s*import\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap(), DependencyType::Import),
        ];
        patterns.insert(Language::JavaScript, js_patterns.clone());
        patterns.insert(Language::TypeScript, js_patterns);
        
        // Java patterns
        patterns.insert(Language::Java, vec![
            (Regex::new(r"^\s*import\s+([a-zA-Z0-9_.]+);").unwrap(), DependencyType::Import),
            (Regex::new(r"^\s*import\s+static\s+([a-zA-Z0-9_.]+);").unwrap(), DependencyType::Import),
        ]);
        
        // Go patterns
        patterns.insert(Language::Go, vec![
            (Regex::new(r#"^\s*import\s+"([^"]+)""#).unwrap(), DependencyType::Import),
            (Regex::new(r"^\s*import\s+\(\s*").unwrap(), DependencyType::Import), // Multi-line imports need special handling
        ]);
        
        // C/C++ patterns
        let c_patterns = vec![
            (Regex::new(r"^\s*#include\s*<([^>]+)>").unwrap(), DependencyType::Include),
            (Regex::new(r#"^\s*#include\s*"([^"]+)""#).unwrap(), DependencyType::Include),
        ];
        patterns.insert(Language::C, c_patterns.clone());
        patterns.insert(Language::Cpp, c_patterns);
        
        Self { patterns }
    }
    
    /// Analyze file content to extract dependencies
    pub fn analyze_file(
        &self,
        content: &str,
        language: &Language,
    ) -> Result<HashSet<Dependency>, GitDomainError> {
        let mut dependencies = HashSet::new();
        
        if let Some(language_patterns) = self.patterns.get(language) {
            for line in content.lines() {
                // Skip comments (simple heuristic)
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
                    continue;
                }
                
                for (pattern, dep_type) in language_patterns {
                    if let Some(captures) = pattern.captures(line) {
                        if let Some(dep_name) = captures.get(1) {
                            dependencies.insert(Dependency {
                                name: dep_name.as_str().to_string(),
                                dependency_type: dep_type.clone(),
                                version: None, // Could be enhanced to extract versions
                            });
                        }
                    }
                }
            }
        }
        
        Ok(dependencies)
    }
    
    /// Analyze package manifest files (Cargo.toml, package.json, etc.)
    pub fn analyze_manifest(
        &self,
        content: &str,
        filename: &str,
    ) -> Result<HashSet<Dependency>, GitDomainError> {
        let mut dependencies = HashSet::new();
        
        match filename {
            "Cargo.toml" => {
                // Simple TOML parsing for dependencies
                let deps_regex = Regex::new(r#"^\s*([a-zA-Z0-9_-]+)\s*=\s*"([^"]+)""#).unwrap();
                let mut in_deps_section = false;
                
                for line in content.lines() {
                    if line.trim() == "[dependencies]" || line.trim() == "[dev-dependencies]" {
                        in_deps_section = true;
                        continue;
                    }
                    if line.trim().starts_with('[') {
                        in_deps_section = false;
                    }
                    
                    if in_deps_section {
                        if let Some(captures) = deps_regex.captures(line) {
                            dependencies.insert(Dependency {
                                name: captures[1].to_string(),
                                dependency_type: DependencyType::Package,
                                version: Some(captures[2].to_string()),
                            });
                        }
                    }
                }
            }
            "package.json" => {
                // Simple JSON parsing for dependencies
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
                    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                        for (name, version) in deps {
                            dependencies.insert(Dependency {
                                name: name.clone(),
                                dependency_type: DependencyType::Package,
                                version: version.as_str().map(std::string::ToString::to_string),
                            });
                        }
                    }
                    if let Some(deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
                        for (name, version) in deps {
                            dependencies.insert(Dependency {
                                name: name.clone(),
                                dependency_type: DependencyType::Package,
                                version: version.as_str().map(std::string::ToString::to_string),
                            });
                        }
                    }
                }
            }
            "requirements.txt" => {
                // Python requirements
                let req_regex = Regex::new(r"^([a-zA-Z0-9_-]+)(?:==|>=|<=|~=|!=)?(.*)$").unwrap();
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        if let Some(captures) = req_regex.captures(trimmed) {
                            dependencies.insert(Dependency {
                                name: captures[1].to_string(),
                                dependency_type: DependencyType::Package,
                                version: if captures[2].is_empty() {
                                    None
                                } else {
                                    Some(captures[2].to_string())
                                },
                            });
                        }
                    }
                }
            }
            "go.mod" => {
                // Go modules
                let mod_regex = Regex::new(r"^\s*require\s+([^\s]+)\s+([^\s]+)").unwrap();
                for line in content.lines() {
                    if let Some(captures) = mod_regex.captures(line) {
                        dependencies.insert(Dependency {
                            name: captures[1].to_string(),
                            dependency_type: DependencyType::Package,
                            version: Some(captures[2].to_string()),
                        });
                    }
                }
            }
            _ => {
                // Unknown manifest type
            }
        }
        
        Ok(dependencies)
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached commit graph entries
    pub commit_graph_entries: usize,
    /// Number of cached dependency graph entries
    pub dependency_graph_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_dependencies() {
        let analyzer = DependencyAnalyzer::new();
        let content = r#"
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
extern crate regex;

fn main() {
    // use inside comment should not be detected
}
"#;
        
        let deps = analyzer.analyze_file(content, &Language::Rust).unwrap();
        assert_eq!(deps.len(), 3);
        assert!(deps.contains(&Dependency {
            name: "std::collections::HashMap".to_string(),
            dependency_type: DependencyType::Use,
            version: None,
        }));
    }
    
    #[test]
    fn test_python_dependencies() {
        let analyzer = DependencyAnalyzer::new();
        let content = r#"
import os
import sys
from collections import defaultdict
from typing import List, Dict

# import in comment
def main():
    pass
"#;
        
        let deps = analyzer.analyze_file(content, &Language::Python).unwrap();
        assert_eq!(deps.len(), 4);
        assert!(deps.contains(&Dependency {
            name: "os".to_string(),
            dependency_type: DependencyType::Import,
            version: None,
        }));
    }
    
    #[test]
    fn test_cargo_toml_parsing() {
        let analyzer = DependencyAnalyzer::new();
        let content = r#"
[package]
name = "test"

[dependencies]
serde = "1.0"
tokio = "1.0"

[dev-dependencies]
criterion = "0.5"
"#;
        
        let deps = analyzer.analyze_manifest(content, "Cargo.toml").unwrap();
        assert_eq!(deps.len(), 3);
        assert!(deps.contains(&Dependency {
            name: "serde".to_string(),
            dependency_type: DependencyType::Package,
            version: Some("1.0".to_string()),
        }));
    }
    
    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("js"), Language::JavaScript);
        assert_eq!(Language::from_extension("java"), Language::Java);
        assert_eq!(Language::from_extension("unknown"), Language::Other("unknown".to_string()));
    }
} 