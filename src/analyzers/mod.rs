// Copyright 2025 Cowboy AI, LLC.

//! Analyzers for extracting graph-focused metadata from Git repositories
//!
//! This module provides analyzers that extract metadata optimized for
//! building graphs with cim-ipld and cim-domain-graphs.

mod collaboration_analyzer;
mod code_quality_analyzer;

pub use collaboration_analyzer::{CollaborationAnalyzer};
pub use code_quality_analyzer::{CodeQualityAnalyzer, FileMetrics};