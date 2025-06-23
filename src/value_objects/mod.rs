//! Value objects for the Git domain
//!
//! Value objects are immutable and represent concepts in the Git domain
//! that are defined by their attributes rather than identity.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a Git commit hash
///
/// Commit hashes must be valid hexadecimal strings with a minimum length
/// of 7 characters (for abbreviated hashes) up to 40 characters for full
/// SHA-1 hashes or 64 characters for SHA-256 hashes.
///
/// # Examples
///
/// ```
/// use cim_domain_git::value_objects::CommitHash;
///
/// // Create from full SHA-1 hash
/// let full_hash = CommitHash::new("1234567890abcdef1234567890abcdef12345678").unwrap();
/// assert_eq!(full_hash.as_str(), "1234567890abcdef1234567890abcdef12345678");
///
/// // Create from abbreviated hash
/// let short_hash = CommitHash::new("abc123def").unwrap();
/// assert_eq!(short_hash.short(), "abc123d");
///
/// // Hashes are normalized to lowercase
/// let mixed_case = CommitHash::new("ABC123DEF").unwrap();
/// assert_eq!(mixed_case.as_str(), "abc123def");
///
/// // Invalid hashes are rejected
/// assert!(CommitHash::new("short").is_err()); // Too short
/// assert!(CommitHash::new("not-hex").is_err()); // Invalid characters
/// assert!(CommitHash::new("").is_err()); // Empty
/// ```
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
///
/// Branch names must follow Git's naming conventions:
/// - Cannot be empty
/// - Cannot contain ".."
/// - Cannot end with "." or "/"
///
/// # Examples
///
/// ```
/// use cim_domain_git::value_objects::BranchName;
///
/// // Valid branch names
/// let main = BranchName::new("main").unwrap();
/// assert!(main.is_default());
///
/// let feature = BranchName::new("feature/new-feature").unwrap();
/// assert!(!feature.is_default());
///
/// let bugfix = BranchName::new("bugfix/issue-123").unwrap();
/// assert_eq!(bugfix.as_str(), "bugfix/issue-123");
///
/// // Invalid branch names
/// assert!(BranchName::new("").is_err()); // Empty
/// assert!(BranchName::new("branch..name").is_err()); // Contains ".."
/// assert!(BranchName::new("branch/").is_err()); // Ends with "/"
/// assert!(BranchName::new("branch.").is_err()); // Ends with "."
/// ```
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

        // Security validation to prevent command injection
        crate::security::validate_branch_name(&name)?;

        // Additional Git branch name restrictions
        if name.ends_with('.') || name.ends_with('/') {
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
///
/// Remote URLs can be in various formats including HTTPS, SSH, and Git protocols.
///
/// # Examples
///
/// ```
/// use cim_domain_git::value_objects::RemoteUrl;
///
/// // HTTPS URLs
/// let https_url = RemoteUrl::new("https://github.com/user/repo.git").unwrap();
/// assert!(https_url.is_github());
/// assert_eq!(https_url.repository_name(), Some("repo"));
///
/// // SSH URLs
/// let ssh_url = RemoteUrl::new("git@github.com:user/repo.git").unwrap();
/// assert!(ssh_url.is_github());
/// assert_eq!(ssh_url.repository_name(), Some("repo"));
///
/// // Git protocol
/// let git_url = RemoteUrl::new("git://github.com/user/repo.git").unwrap();
/// assert!(git_url.is_github());
///
/// // Repository name extraction works with or without .git suffix
/// let no_suffix = RemoteUrl::new("https://github.com/user/repository").unwrap();
/// assert_eq!(no_suffix.repository_name(), Some("repository"));
///
/// // Invalid URLs
/// assert!(RemoteUrl::new("").is_err());
/// assert!(RemoteUrl::new("not-a-url").is_err());
/// ```
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

        // Security validation to prevent command injection
        crate::security::validate_remote_url(&url)?;

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
///
/// Represents the author or committer of a Git commit.
///
/// # Examples
///
/// ```
/// use cim_domain_git::value_objects::AuthorInfo;
///
/// // Create author info
/// let author = AuthorInfo::new("Jane Doe", "jane@example.com");
/// assert_eq!(author.name, "Jane Doe");
/// assert_eq!(author.email, "jane@example.com");
///
/// // Display format follows Git convention
/// assert_eq!(author.to_string(), "Jane Doe <jane@example.com>");
///
/// // Can be created with any string types
/// let author2 = AuthorInfo::new(
///     String::from("John Smith"),
///     String::from("john@example.com")
/// );
/// assert_eq!(author2.name, "John Smith");
/// ```
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
///
/// File paths are normalized to use forward slashes regardless of platform.
///
/// # Examples
///
/// ```
/// use cim_domain_git::value_objects::FilePath;
///
/// // Create a file path
/// let path = FilePath::new("src/lib.rs").unwrap();
/// assert_eq!(path.as_str(), "src/lib.rs");
/// assert_eq!(path.file_name(), Some("lib.rs"));
/// assert_eq!(path.directory(), Some("src"));
/// assert_eq!(path.extension(), Some("rs"));
///
/// // Paths are normalized
/// let windows_path = FilePath::new("src\\main\\java\\App.java").unwrap();
/// assert_eq!(windows_path.as_str(), "src/main/java/App.java");
///
/// // Complex paths
/// let complex = FilePath::new("path/to/file.tar.gz").unwrap();
/// assert_eq!(complex.file_name(), Some("file.tar.gz"));
/// assert_eq!(complex.extension(), Some("gz"));
///
/// // Root level files
/// let root_file = FilePath::new("README.md").unwrap();
/// assert_eq!(root_file.file_name(), Some("README.md"));
/// assert_eq!(root_file.directory(), None);
///
/// // Invalid paths
/// assert!(FilePath::new("").is_err());
/// ```
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

        // Security validation to prevent path traversal
        let validated_path = crate::security::validate_path(&path)?;

        // Normalize path separators
        let normalized = validated_path.to_string_lossy().replace('\\', "/");

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
