// Copyright 2025 Cowboy AI, LLC.

//! Comprehensive tests for value objects

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_remote_url_valid() {
        // HTTPS URLs
        assert!(RemoteUrl::new("https://github.com/user/repo.git").is_ok());
        assert!(RemoteUrl::new("https://gitlab.com/user/repo.git").is_ok());
        assert!(RemoteUrl::new("https://bitbucket.org/user/repo.git").is_ok());
        
        // HTTP URLs
        assert!(RemoteUrl::new("http://github.com/user/repo.git").is_ok());
        
        // SSH URLs
        assert!(RemoteUrl::new("ssh://git@github.com/user/repo.git").is_ok());
        assert!(RemoteUrl::new("git@github.com:user/repo.git").is_ok());
        
        // Git protocol
        assert!(RemoteUrl::new("git://github.com/user/repo.git").is_ok());
        
        // File paths
        assert!(RemoteUrl::new("file:///path/to/repo").is_ok());
        assert!(RemoteUrl::new("/absolute/path/to/repo").is_ok());
        assert!(RemoteUrl::new("../relative/path/to/repo").is_ok());
    }

    #[test]
    fn test_remote_url_invalid() {
        assert!(RemoteUrl::new("").is_err());
        assert!(RemoteUrl::new("not a url").is_err());
        assert!(RemoteUrl::new("ftp://invalid.com/repo").is_err());
        assert!(RemoteUrl::new("://no-protocol.com").is_err());
    }

    #[test]
    fn test_commit_hash_valid() {
        // Short hashes (minimum 7 characters)
        assert!(CommitHash::new("abc123d").is_ok());
        assert!(CommitHash::new("1234567").is_ok());
        
        // Full SHA-1 hash
        assert!(CommitHash::new("abcdef1234567890abcdef1234567890abcdef12").is_ok());
        
        // Mixed case (should be normalized to lowercase)
        let hash = CommitHash::new("AbC123D").unwrap();
        assert_eq!(hash.as_str(), "abc123d");
    }

    #[test]
    fn test_commit_hash_invalid() {
        assert!(CommitHash::new("").is_err());
        assert!(CommitHash::new("abc").is_err()); // Too short
        assert!(CommitHash::new("xyz123").is_err()); // Invalid characters
        assert!(CommitHash::new("g123456").is_err()); // 'g' is not hex
        assert!(CommitHash::new("12345678901234567890123456789012345678901").is_err()); // Too long
    }

    #[test]
    fn test_branch_name_valid() {
        assert!(BranchName::new("main").is_ok());
        assert!(BranchName::new("feature/new-feature").is_ok());
        assert!(BranchName::new("bugfix/issue-123").is_ok());
        assert!(BranchName::new("release/v1.0.0").is_ok());
        assert!(BranchName::new("user/john-doe/feature").is_ok());
        assert!(BranchName::new("feature_branch").is_ok());
        assert!(BranchName::new("feature-branch").is_ok());
        assert!(BranchName::new("UPPERCASE").is_ok());
        assert!(BranchName::new("with.dots").is_ok());
    }

    #[test]
    fn test_branch_name_invalid() {
        assert!(BranchName::new("").is_err());
        assert!(BranchName::new(".hidden").is_err()); // Starts with dot
        assert!(BranchName::new("ends.").is_err()); // Ends with dot
        assert!(BranchName::new("double..dot").is_err()); // Consecutive dots
        assert!(BranchName::new("/leading-slash").is_err());
        assert!(BranchName::new("trailing-slash/").is_err());
        assert!(BranchName::new("double//slash").is_err());
        assert!(BranchName::new("space branch").is_err());
        assert!(BranchName::new("branch~tilde").is_err());
        assert!(BranchName::new("branch^caret").is_err());
        assert!(BranchName::new("branch:colon").is_err());
        assert!(BranchName::new("branch?question").is_err());
        assert!(BranchName::new("branch*asterisk").is_err());
        assert!(BranchName::new("branch[bracket").is_err());
        assert!(BranchName::new("branch\\backslash").is_err());
        assert!(BranchName::new("branch..lock").is_err()); // Ends with .lock
        assert!(BranchName::new("@").is_err()); // Single @
        assert!(BranchName::new("branch@{").is_err()); // Contains @{
    }

    #[test]
    fn test_tag_name_valid() {
        assert!(TagName::new("v1.0.0").is_ok());
        assert!(TagName::new("release-1.0.0").is_ok());
        assert!(TagName::new("2024.01.01").is_ok());
        assert!(TagName::new("alpha").is_ok());
        assert!(TagName::new("beta-1").is_ok());
        assert!(TagName::new("v1.0.0-rc1").is_ok());
    }

    #[test]
    fn test_tag_name_invalid() {
        // Same rules as branch names
        assert!(TagName::new("").is_err());
        assert!(TagName::new(".hidden").is_err());
        assert!(TagName::new("tag with space").is_err());
        assert!(TagName::new("tag~tilde").is_err());
    }

    #[test]
    fn test_file_path_valid() {
        assert!(FilePath::new("src/main.rs").is_ok());
        assert!(FilePath::new("README.md").is_ok());
        assert!(FilePath::new("path/to/file.txt").is_ok());
        assert!(FilePath::new("src/deeply/nested/path/file.rs").is_ok());
        assert!(FilePath::new(".gitignore").is_ok());
        assert!(FilePath::new("file-with-dash.txt").is_ok());
        assert!(FilePath::new("file_with_underscore.txt").is_ok());
        assert!(FilePath::new("file.multiple.dots.txt").is_ok());
        assert!(FilePath::new("文件.txt").is_ok()); // Unicode
    }

    #[test]
    fn test_file_path_invalid() {
        assert!(FilePath::new("").is_err());
        assert!(FilePath::new("/absolute/path").is_err()); // No absolute paths
        assert!(FilePath::new("../parent/path").is_err()); // No parent directory
        assert!(FilePath::new("./current/path").is_err()); // No current directory
        assert!(FilePath::new("path/../file").is_err()); // No parent in middle
        assert!(FilePath::new("path/./file").is_err()); // No current in middle
        assert!(FilePath::new("path//double/slash").is_err());
        assert!(FilePath::new("path/").is_err()); // Trailing slash
        assert!(FilePath::new("\0null").is_err()); // Null byte
    }

    #[test]
    fn test_commit_message_valid() {
        assert!(CommitMessage::new("Fix bug in parser").is_ok());
        assert!(CommitMessage::new("feat: Add new feature\n\nDetailed description").is_ok());
        
        // Multiline messages
        let long_message = "feat: Add comprehensive logging\n\n\
            This commit adds logging throughout the application:\n\
            - Added debug logs in parser\n\
            - Added info logs in service layer\n\
            - Added error logs with context";
        assert!(CommitMessage::new(long_message).is_ok());
    }

    #[test]
    fn test_commit_message_invalid() {
        assert!(CommitMessage::new("").is_err());
        assert!(CommitMessage::new("   ").is_err()); // Only whitespace
        assert!(CommitMessage::new("\n\n").is_err()); // Only newlines
    }

    #[test]
    fn test_commit_message_summary() {
        let msg = CommitMessage::new("feat: Add new feature\n\nDetailed description").unwrap();
        assert_eq!(msg.summary(), "feat: Add new feature");
        
        let msg = CommitMessage::new("Single line message").unwrap();
        assert_eq!(msg.summary(), "Single line message");
        
        // Long first line should be truncated
        let long_first_line = "a".repeat(100);
        let msg = CommitMessage::new(&format!("{}\n\nBody", long_first_line)).unwrap();
        assert!(msg.summary().len() <= 72);
    }

    #[test]
    fn test_commit_message_body() {
        let msg = CommitMessage::new("Subject\n\nBody line 1\nBody line 2").unwrap();
        assert_eq!(msg.body(), Some("Body line 1\nBody line 2"));
        
        let msg = CommitMessage::new("Only subject").unwrap();
        assert_eq!(msg.body(), None);
        
        let msg = CommitMessage::new("Subject\n\n").unwrap();
        assert_eq!(msg.body(), None); // Empty body
    }

    #[test]
    fn test_author_info() {
        let author = AuthorInfo {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };
        
        assert_eq!(author.name, "John Doe");
        assert_eq!(author.email, "john@example.com");
    }

    #[test]
    fn test_value_object_serialization() {
        // Test RemoteUrl
        let url = RemoteUrl::new("https://github.com/user/repo.git").unwrap();
        let json = serde_json::to_string(&url).unwrap();
        let deserialized: RemoteUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(url, deserialized);
        
        // Test CommitHash
        let hash = CommitHash::new("abc123def").unwrap();
        let json = serde_json::to_string(&hash).unwrap();
        let deserialized: CommitHash = serde_json::from_str(&json).unwrap();
        assert_eq!(hash, deserialized);
        
        // Test BranchName
        let branch = BranchName::new("feature/test").unwrap();
        let json = serde_json::to_string(&branch).unwrap();
        let deserialized: BranchName = serde_json::from_str(&json).unwrap();
        assert_eq!(branch, deserialized);
        
        // Test FilePath
        let path = FilePath::new("src/main.rs").unwrap();
        let json = serde_json::to_string(&path).unwrap();
        let deserialized: FilePath = serde_json::from_str(&json).unwrap();
        assert_eq!(path, deserialized);
    }

    #[test]
    fn test_value_object_display() {
        let url = RemoteUrl::new("https://github.com/user/repo.git").unwrap();
        assert_eq!(url.to_string(), "https://github.com/user/repo.git");
        
        let hash = CommitHash::new("abc123").unwrap();
        assert_eq!(hash.to_string(), "abc123");
        
        let branch = BranchName::new("main").unwrap();
        assert_eq!(branch.to_string(), "main");
        
        let tag = TagName::new("v1.0.0").unwrap();
        assert_eq!(tag.to_string(), "v1.0.0");
        
        let path = FilePath::new("README.md").unwrap();
        assert_eq!(path.to_string(), "README.md");
        
        let msg = CommitMessage::new("Initial commit").unwrap();
        assert_eq!(msg.to_string(), "Initial commit");
    }

    #[test]
    fn test_value_object_as_str() {
        let hash = CommitHash::new("abc123def").unwrap();
        assert_eq!(hash.as_str(), "abc123def");
        
        let branch = BranchName::new("main").unwrap();
        assert_eq!(branch.as_str(), "main");
        
        let tag = TagName::new("v1.0.0").unwrap();
        assert_eq!(tag.as_str(), "v1.0.0");
        
        let path = FilePath::new("src/lib.rs").unwrap();
        assert_eq!(path.as_str(), "src/lib.rs");
    }
}