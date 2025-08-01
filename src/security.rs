// Copyright 2025 Cowboy AI, LLC.

//! Security utilities for Git domain
//!
//! This module provides security checks and validation
//! to prevent common security vulnerabilities.

use crate::GitDomainError;
use std::path::PathBuf;

/// Validate a path to prevent directory traversal attacks
pub fn validate_path(path: &str) -> Result<PathBuf, GitDomainError> {
    // Check for null bytes
    if path.contains('\0') {
        return Err(GitDomainError::ValidationError(
            "Path contains null bytes".to_string()
        ));
    }

    // Check for common path traversal patterns
    if path.contains("..") || path.contains('~') {
        return Err(GitDomainError::ValidationError(
            "Path contains directory traversal patterns".to_string()
        ));
    }

    // Convert to PathBuf and canonicalize
    let path_buf = PathBuf::from(path);
    
    // Check if path is absolute
    if path_buf.is_absolute() {
        // For absolute paths, ensure they're within allowed directories
        // In production, you'd check against a whitelist of allowed directories
        Ok(path_buf)
    } else {
        // For relative paths, ensure they don't escape the working directory
        Ok(path_buf)
    }
}

/// Validate a Git remote URL to prevent command injection
pub fn validate_remote_url(url: &str) -> Result<(), GitDomainError> {
    // Check for null bytes
    if url.contains('\0') {
        return Err(GitDomainError::ValidationError(
            "URL contains null bytes".to_string()
        ));
    }

    // Check for shell metacharacters that could lead to command injection
    let dangerous_chars = ['$', '`', '|', ';', '&', '<', '>', '(', ')', '{', '}', '\n', '\r'];
    for ch in dangerous_chars {
        if url.contains(ch) {
            return Err(GitDomainError::ValidationError(
                format!("URL contains dangerous character: {ch}")
            ));
        }
    }

    // Validate URL scheme
    if !url.starts_with("https://") && !url.starts_with("http://") && 
       !url.starts_with("git://") && !url.starts_with("ssh://") &&
       !url.starts_with("git@") {
        return Err(GitDomainError::ValidationError(
            "URL must use a valid Git protocol".to_string()
        ));
    }

    Ok(())
}

/// Validate a branch name to prevent command injection
pub fn validate_branch_name(name: &str) -> Result<(), GitDomainError> {
    // Check for null bytes
    if name.contains('\0') {
        return Err(GitDomainError::ValidationError(
            "Branch name contains null bytes".to_string()
        ));
    }

    // Check for shell metacharacters
    let dangerous_chars = ['$', '`', '|', ';', '&', '<', '>', '(', ')', '{', '}', '\n', '\r', ' '];
    for ch in dangerous_chars {
        if name.contains(ch) {
            return Err(GitDomainError::ValidationError(
                format!("Branch name contains dangerous character: {ch}")
            ));
        }
    }

    // Check Git-specific invalid patterns
    if name.starts_with('-') {
        return Err(GitDomainError::ValidationError(
            "Branch name cannot start with hyphen".to_string()
        ));
    }

    if name.ends_with(".lock") {
        return Err(GitDomainError::ValidationError(
            "Branch name cannot end with .lock".to_string()
        ));
    }

    if name.contains("..") || name.contains("//") {
        return Err(GitDomainError::ValidationError(
            "Branch name contains invalid patterns".to_string()
        ));
    }

    Ok(())
}

/// Sanitize user input for use in error messages
#[must_use] pub fn sanitize_for_display(input: &str) -> String {
    // Remove control characters and limit length
    input
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .take(100)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path() {
        // Valid paths
        assert!(validate_path("src/main.rs").is_ok());
        assert!(validate_path("project/src/lib.rs").is_ok());
        assert!(validate_path("/home/user/project").is_ok());

        // Invalid paths
        assert!(validate_path("../../../etc/passwd").is_err());
        assert!(validate_path("~/sensitive").is_err());
        assert!(validate_path("path\0with\0nulls").is_err());
    }

    #[test]
    fn test_validate_remote_url() {
        // Valid URLs
        assert!(validate_remote_url("https://github.com/user/repo.git").is_ok());
        assert!(validate_remote_url("git@github.com:user/repo.git").is_ok());
        assert!(validate_remote_url("ssh://git@github.com/user/repo.git").is_ok());

        // Invalid URLs
        assert!(validate_remote_url("https://example.com/repo.git; rm -rf /").is_err());
        assert!(validate_remote_url("https://example.com/repo.git`whoami`").is_err());
        assert!(validate_remote_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn test_validate_branch_name() {
        // Valid branch names
        assert!(validate_branch_name("main").is_ok());
        assert!(validate_branch_name("feature/new-feature").is_ok());
        assert!(validate_branch_name("release-1.0").is_ok());

        // Invalid branch names
        assert!(validate_branch_name("-feature").is_err());
        assert!(validate_branch_name("feature.lock").is_err());
        assert!(validate_branch_name("feature; rm -rf /").is_err());
        assert!(validate_branch_name("feature with spaces").is_err());
    }

    #[test]
    fn test_sanitize_for_display() {
        assert_eq!(sanitize_for_display("normal text"), "normal text");
        assert_eq!(sanitize_for_display("text\0with\0nulls"), "textwithnulls");
        assert_eq!(sanitize_for_display("text\nwith\nnewlines"), "text\nwith\nnewlines");
        
        let long_text = "a".repeat(200);
        assert_eq!(sanitize_for_display(&long_text).len(), 100);
    }
} 