//! Value objects for the Git domain
//!
//! Value objects are immutable and represent concepts in the Git domain
//! that are defined by their attributes rather than identity.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a Git commit hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitHash(String);

impl CommitHash {
    /// Create a new commit hash
    pub fn new(hash: impl Into<String>) -> Result<Self, crate::GitDomainError> {
        let hash = hash.into();

        // Basic validation - Git hashes are hexadecimal
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(crate::GitDomainError::InvalidCommitHash(hash));
        }

        // SHA-1 is 40 chars, SHA-256 is 64 chars, but we also accept short hashes
        if hash.len() < 7 {
            return Err(crate::GitDomainError::InvalidCommitHash(
                "Commit hash too short".to_string(),
            ));
        }

        Ok(Self(hash.to_lowercase()))
    }

    /// Get the hash as a string
    #[must_use] pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get a short version of the hash (first 7 characters)
    #[must_use] pub fn short(&self) -> &str {
        &self.0[..7.min(self.0.len())]
    }
}

impl fmt::Display for CommitHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A Git branch name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchName(String);

impl BranchName {
    /// Create a new branch name
    pub fn new(name: impl Into<String>) -> Result<Self, crate::GitDomainError> {
        let name = name.into();

        // Basic validation for branch names
        if name.is_empty() {
            return Err(crate::GitDomainError::GitOperationFailed(
                "Branch name cannot be empty".to_string(),
            ));
        }

        // Git branch name restrictions
        if name.contains("..") || name.ends_with('.') || name.ends_with('/') {
            return Err(crate::GitDomainError::GitOperationFailed(format!(
                "Invalid branch name: {name}"
            )));
        }

        Ok(Self(name))
    }

    /// Get the branch name as a string
    #[must_use] pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this is the main/master branch
    #[must_use] pub fn is_default(&self) -> bool {
        matches!(self.0.as_str(), "main" | "master")
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A Git remote URL
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RemoteUrl(String);

impl RemoteUrl {
    /// Create a new remote URL
    pub fn new(url: impl Into<String>) -> Result<Self, crate::GitDomainError> {
        let url = url.into();

        // Basic URL validation
        if url.is_empty() {
            return Err(crate::GitDomainError::GitOperationFailed(
                "Remote URL cannot be empty".to_string(),
            ));
        }

        // Check for common Git URL patterns
        if !url.starts_with("http://")
            && !url.starts_with("https://")
            && !url.starts_with("git://")
            && !url.starts_with("ssh://")
            && !url.contains('@')
        // git@github.com:user/repo.git
        {
            return Err(crate::GitDomainError::GitOperationFailed(format!(
                "Invalid Git remote URL: {url}"
            )));
        }

        Ok(Self(url))
    }

    /// Get the URL as a string
    #[must_use] pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extract the repository name from the URL
    #[must_use] pub fn repository_name(&self) -> Option<&str> {
        self.0
            .split('/')
            .next_back()
            .map(|name| name.trim_end_matches(".git"))
    }

    /// Check if this is a GitHub URL
    #[must_use] pub fn is_github(&self) -> bool {
        self.0.contains("github.com")
    }
}

impl fmt::Display for RemoteUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Git author information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthorInfo {
    /// Author name
    pub name: String,

    /// Author email
    pub email: String,
}

impl AuthorInfo {
    /// Create new author info
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
        }
    }
}

impl fmt::Display for AuthorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

/// A Git tag name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagName(String);

impl TagName {
    /// Create a new tag name
    pub fn new(name: impl Into<String>) -> Result<Self, crate::GitDomainError> {
        let name = name.into();

        if name.is_empty() {
            return Err(crate::GitDomainError::GitOperationFailed(
                "Tag name cannot be empty".to_string(),
            ));
        }

        Ok(Self(name))
    }

    /// Get the tag name as a string
    #[must_use] pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this looks like a semantic version tag
    #[must_use] pub fn is_semver(&self) -> bool {
        self.0.starts_with('v') && self.0[1..].chars().next().is_some_and(char::is_numeric)
    }
}

impl fmt::Display for TagName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// File path within a Git repository
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FilePath(String);

impl FilePath {
    /// Create a new file path
    pub fn new(path: impl Into<String>) -> Result<Self, crate::GitDomainError> {
        let path = path.into();

        if path.is_empty() {
            return Err(crate::GitDomainError::GitOperationFailed(
                "File path cannot be empty".to_string(),
            ));
        }

        // Normalize path separators
        let normalized = path.replace('\\', "/");

        Ok(Self(normalized))
    }

    /// Get the path as a string
    #[must_use] pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the file name (last component)
    #[must_use] pub fn file_name(&self) -> Option<&str> {
        self.0.split('/').next_back()
    }

    /// Get the directory path
    #[must_use] pub fn directory(&self) -> Option<&str> {
        self.0.rfind('/').map(|idx| &self.0[..idx])
    }

    /// Get the file extension
    #[must_use] pub fn extension(&self) -> Option<&str> {
        self.file_name()
            .and_then(|name| name.rfind('.'))
            .and_then(|idx| {
                let name = self.file_name()?;
                Some(&name[idx + 1..])
            })
    }
}

impl fmt::Display for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_hash_validation() {
        // Valid hashes
        assert!(CommitHash::new("abc123def456").is_ok());
        assert!(CommitHash::new("1234567890abcdef1234567890abcdef12345678").is_ok());

        // Invalid hashes
        assert!(CommitHash::new("short").is_err());
        assert!(CommitHash::new("not-hex-chars").is_err());
        assert!(CommitHash::new("").is_err());
    }

    #[test]
    fn test_branch_name_validation() {
        // Valid names
        assert!(BranchName::new("main").is_ok());
        assert!(BranchName::new("feature/new-feature").is_ok());

        // Invalid names
        assert!(BranchName::new("").is_err());
        assert!(BranchName::new("branch..name").is_err());
        assert!(BranchName::new("branch/").is_err());
    }

    #[test]
    fn test_remote_url_parsing() {
        let url = RemoteUrl::new("https://github.com/user/repo.git").unwrap();
        assert_eq!(url.repository_name(), Some("repo"));
        assert!(url.is_github());

        let ssh_url = RemoteUrl::new("git@github.com:user/repo.git").unwrap();
        assert!(ssh_url.is_github());
    }

    #[test]
    fn test_file_path_parsing() {
        let path = FilePath::new("src/lib.rs").unwrap();
        assert_eq!(path.file_name(), Some("lib.rs"));
        assert_eq!(path.directory(), Some("src"));
        assert_eq!(path.extension(), Some("rs"));
    }
}
