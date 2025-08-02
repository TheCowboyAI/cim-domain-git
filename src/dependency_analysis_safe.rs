// Copyright 2025 Cowboy AI, LLC.

//! Safe dependency analysis implementation with lazy_static

use crate::dependency_analysis::{Dependency, DependencyType, Language};
use crate::GitDomainError;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};

lazy_static! {
    // Rust patterns
    static ref RUST_USE: Regex = Regex::new(r"^\s*use\s+([a-zA-Z0-9_:]+)").expect("Invalid regex");
    static ref RUST_EXTERN: Regex = Regex::new(r"^\s*extern\s+crate\s+([a-zA-Z0-9_]+)").expect("Invalid regex");
    
    // Python patterns
    static ref PYTHON_IMPORT: Regex = Regex::new(r"^\s*import\s+([a-zA-Z0-9_.]+)").expect("Invalid regex");
    static ref PYTHON_FROM: Regex = Regex::new(r"^\s*from\s+([a-zA-Z0-9_.]+)\s+import").expect("Invalid regex");
    
    // JavaScript/TypeScript patterns
    static ref JS_IMPORT: Regex = Regex::new(r#"^\s*import\s+.*\s+from\s+['"]([^'"]+)['"]"#).expect("Invalid regex");
    static ref JS_REQUIRE: Regex = Regex::new(r#"^\s*const\s+.*\s*=\s*require\s*\(\s*['"]([^'"]+)['"]\s*\)"#).expect("Invalid regex");
    
    // Go patterns
    static ref GO_IMPORT: Regex = Regex::new(r#"^\s*import\s+"([^"]+)""#).expect("Invalid regex");
    static ref GO_IMPORT_PAREN: Regex = Regex::new(r"^\s*import\s+\(\s*").expect("Invalid regex");
    
    // Java patterns
    static ref JAVA_IMPORT: Regex = Regex::new(r"^\s*import\s+([a-zA-Z0-9_.]+);").expect("Invalid regex");
    static ref JAVA_STATIC: Regex = Regex::new(r"^\s*import\s+static\s+([a-zA-Z0-9_.]+);").expect("Invalid regex");
    
    // C/C++ patterns
    static ref C_INCLUDE_SYS: Regex = Regex::new(r"^\s*#include\s*<([^>]+)>").expect("Invalid regex");
    static ref C_INCLUDE_LOCAL: Regex = Regex::new(r#"^\s*#include\s*"([^"]+)""#).expect("Invalid regex");
    
    // Manifest file patterns
    static ref CARGO_DEPS: Regex = Regex::new(r#"^\s*([a-zA-Z0-9_-]+)\s*=\s*"([^"]+)""#).expect("Invalid regex");
    static ref PIP_REQ: Regex = Regex::new(r"^([a-zA-Z0-9_-]+)(?:==|>=|<=|~=|!=)?(.*)$").expect("Invalid regex");
    static ref GO_MOD: Regex = Regex::new(r"^\s*require\s+([^\s]+)\s+([^\s]+)").expect("Invalid regex");
}

/// Get patterns for a specific language
fn get_patterns(language: &Language) -> Vec<(&'static Regex, DependencyType)> {
    match language {
        Language::Rust => vec![
            (&*RUST_USE, DependencyType::Use),
            (&*RUST_EXTERN, DependencyType::Use),
        ],
        Language::Python => vec![
            (&*PYTHON_IMPORT, DependencyType::Import),
            (&*PYTHON_FROM, DependencyType::Import),
        ],
        Language::JavaScript | Language::TypeScript => vec![
            (&*JS_IMPORT, DependencyType::Import),
            (&*JS_REQUIRE, DependencyType::Import),
        ],
        Language::Go => vec![
            (&*GO_IMPORT, DependencyType::Import),
            // GO_IMPORT_PAREN needs special handling for multi-line
        ],
        Language::Java => vec![
            (&*JAVA_IMPORT, DependencyType::Import),
            (&*JAVA_STATIC, DependencyType::Import),
        ],
        Language::C | Language::Cpp => vec![
            (&*C_INCLUDE_SYS, DependencyType::Include),
            (&*C_INCLUDE_LOCAL, DependencyType::Include),
        ],
        Language::Other(_) => vec![],
    }
}

/// Analyze file content to extract dependencies
pub fn analyze_file_safe(
    content: &str,
    language: &Language,
) -> Result<HashSet<Dependency>, GitDomainError> {
    let mut dependencies = HashSet::new();
    let patterns = get_patterns(language);

    for line in content.lines() {
        // Skip comments (simple heuristic)
        let trimmed = line.trim();
        if trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
        {
            continue;
        }

        for (pattern, dep_type) in &patterns {
            if let Some(captures) = pattern.captures(line) {
                if let Some(dep_name) = captures.get(1) {
                    dependencies.insert(Dependency {
                        name: dep_name.as_str().to_string(),
                        dependency_type: dep_type.clone(),
                        version: None,
                    });
                }
            }
        }
    }

    Ok(dependencies)
}

/// Analyze package manifest files safely
pub fn analyze_manifest_safe(
    content: &str,
    filename: &str,
) -> Result<HashSet<Dependency>, GitDomainError> {
    let mut dependencies = HashSet::new();

    match filename {
        "Cargo.toml" => {
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
                    if let Some(captures) = CARGO_DEPS.captures(line) {
                        dependencies.insert(Dependency {
                            name: captures[1].to_string(),
                            dependency_type: DependencyType::Package,
                            version: Some(captures[2].to_string()),
                        });
                    }
                }
            }
        }
        "requirements.txt" => {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    if let Some(captures) = PIP_REQ.captures(trimmed) {
                        dependencies.insert(Dependency {
                            name: captures[1].to_string(),
                            dependency_type: DependencyType::Package,
                            version: captures.get(2).map(|m| m.as_str().trim().to_string()),
                        });
                    }
                }
            }
        }
        "go.mod" => {
            for line in content.lines() {
                if let Some(captures) = GO_MOD.captures(line) {
                    dependencies.insert(Dependency {
                        name: captures[1].to_string(),
                        dependency_type: DependencyType::Package,
                        version: Some(captures[2].to_string()),
                    });
                }
            }
        }
        "package.json" => {
            // For package.json, we'd ideally use a JSON parser
            // This is a simplified version
            let mut in_deps = false;
            for line in content.lines() {
                if line.contains("\"dependencies\"") || line.contains("\"devDependencies\"") {
                    in_deps = true;
                    continue;
                }
                if in_deps && line.contains('}') {
                    in_deps = false;
                }
                if in_deps && line.contains(':') {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        let name = parts[0].trim().trim_matches('"');
                        let version = parts[1].trim().trim_matches('"').trim_matches(',');
                        dependencies.insert(Dependency {
                            name: name.to_string(),
                            dependency_type: DependencyType::Package,
                            version: Some(version.to_string()),
                        });
                    }
                }
            }
        }
        _ => {}
    }

    Ok(dependencies)
}