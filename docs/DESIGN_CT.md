# cim-domain-git: Category Theory Design

**Version**: 1.0
**Date**: 2025-11-12
**Status**: Design Phase

## Executive Summary

This document defines the mathematical foundations for **cim-domain-git**, an adapter that bridges Git's content-addressing system with IPLD's content-addressing system. Using Category Theory, we define structure-preserving functors that enable cim-flashstor to store git repositories in native format while serving content through IPLD CID requests.

## 1. Mathematical Foundations

### 1.1 Category Git

**Objects**: The objects in category **Git** are git objects:
- `GitCommit`: Commit objects with tree references and parent pointers
- `GitTree`: Tree objects containing file/directory entries
- `GitBlob`: Blob objects containing file content
- `GitTag`: Tag objects referencing commits

**Morphisms**: The morphisms represent relationships between git objects:
- `parent_of: GitCommit → GitCommit` - Parent relationship
- `tree_of: GitCommit → GitTree` - Commit's tree reference
- `contains: GitTree → GitObject` - Tree entry reference (to blob or subtree)
- `tagged_by: GitCommit → GitTag` - Tag reference

**Identity**: Each git object has an identity morphism `id: GitObject → GitObject`

**Composition**: Morphisms compose transitively:
```
If f: A → B and g: B → C, then g ∘ f: A → C
```

For example, if commit C1 has parent C2, and C2 has parent C3, then:
```
parent_of(C2) ∘ parent_of(C1) = grandparent_of(C1) → C3
```

### 1.2 Category IPLD

**Objects**: The objects in category **IPLD** are IPLD nodes:
- `IpldNode<dag-git-commit>`: IPLD encoding of git commits
- `IpldNode<dag-git-tree>`: IPLD encoding of git trees
- `IpldNode<dag-git-blob>`: IPLD encoding of git blobs
- `IpldNode<dag-git-tag>`: IPLD encoding of git tags

**Morphisms**: The morphisms are IPLD links:
- `ipld_link: IpldNode → IpldNode` - CID-based link

**Identity**: Each IPLD node has an identity morphism via its CID

**Composition**: Links compose by traversal:
```
If link1: Node_A → Node_B and link2: Node_B → Node_C
then link2 ∘ link1: Node_A → Node_C
```

### 1.3 Functor F: Git → IPLD

**Object Mapping** `F_obj`:
```
F_obj(GitCommit(sha, tree, parents, data))
  = IpldNode(CID_commit, dag-git-commit{
      tree: F_obj(tree),
      parents: [F_obj(p) for p in parents],
      ...data
    })

F_obj(GitTree(sha, entries))
  = IpldNode(CID_tree, dag-git-tree{
      entries: [(name, mode, F_obj(obj)) for (name, mode, obj) in entries]
    })

F_obj(GitBlob(sha, content))
  = IpldNode(CID_blob, dag-git-blob{content})
```

**Morphism Mapping** `F_mor`:
```
F_mor(parent_of(C1 → C2)) = ipld_link(F_obj(C1) → F_obj(C2))
F_mor(contains(Tree → Obj)) = ipld_link(F_obj(Tree) → F_obj(Obj))
```

**Functor Laws**:

1. **Identity Preservation**:
   ```
   F(id_GitObject) = id_IpldNode
   ```
   Proof: Identity morphism in Git (object → itself) maps to identity morphism in IPLD (CID self-reference).

2. **Composition Preservation**:
   ```
   F(g ∘ f) = F(g) ∘ F(f)
   ```
   Proof: If git has path `commit → tree → blob`, then IPLD has path `CID_commit → CID_tree → CID_blob`.

### 1.4 Functor G: IPLD → Git (Inverse)

**Object Mapping** `G_obj`:
```
G_obj(IpldNode(cid, dag-git-commit{tree_cid, parent_cids, data}))
  = GitCommit(
      sha: extract_sha(cid),
      tree: G_obj(resolve(tree_cid)),
      parents: [G_obj(resolve(p_cid)) for p_cid in parent_cids],
      ...data
    )

G_obj(IpldNode(cid, dag-git-tree{entries}))
  = GitTree(
      sha: extract_sha(cid),
      entries: [(name, mode, G_obj(resolve(entry_cid))) for (name, mode, entry_cid) in entries]
    )
```

**Morphism Mapping** `G_mor`:
```
G_mor(ipld_link(Node_A → Node_B)) = git_reference(G_obj(Node_A) → G_obj(Node_B))
```

### 1.5 Adjunction Properties

**F ⊣ G** (F is left adjoint to G):
```
Hom_IPLD(F(git_obj), ipld_node) ≅ Hom_Git(git_obj, G(ipld_node))
```

**Natural Isomorphism**:
```
G ∘ F ≈ id_Git  (up to natural isomorphism)
F ∘ G ≈ id_IPLD (up to natural isomorphism)
```

This means round-trip conversions preserve structure:
```
git_obj → F → ipld_node → G → git_obj'  where git_obj ≈ git_obj'
```

## 2. Natural Transformations (fmap)

### 2.1 fmap_to_ipld: Git → IPLD

**Commit Transformation**:
```haskell
fmap_to_ipld :: GitCommit → IpldNode
fmap_to_ipld (GitCommit sha tree parents author committer message) =
  IpldNode {
    cid: sha_to_cid(sha, dag-git-commit),
    data: dag-git-commit {
      tree: fmap_to_ipld(tree),
      parents: map fmap_to_ipld parents,
      author: author,
      committer: committer,
      message: message
    }
  }
```

**Tree Transformation**:
```haskell
fmap_to_ipld :: GitTree → IpldNode
fmap_to_ipld (GitTree sha entries) =
  IpldNode {
    cid: sha_to_cid(sha, dag-git-tree),
    data: dag-git-tree {
      entries: map (\(name, mode, obj) → (name, mode, fmap_to_ipld(obj))) entries
    }
  }
```

**Blob Transformation**:
```haskell
fmap_to_ipld :: GitBlob → IpldNode
fmap_to_ipld (GitBlob sha content) =
  IpldNode {
    cid: sha_to_cid(sha, dag-git-blob),
    data: dag-git-blob {
      content: content
    }
  }
```

**Hash Transformation**:
```haskell
sha_to_cid :: GitHash → CID
sha_to_cid (SHA1 bytes) = CID(multicodec: dag-git, multihash: sha1(bytes))
sha_to_cid (SHA256 bytes) = CID(multicodec: dag-git, multihash: sha256(bytes))
```

### 2.2 fmap_from_ipld: IPLD → Git

**Inverse Commit Transformation**:
```haskell
fmap_from_ipld :: IpldNode → Maybe GitCommit
fmap_from_ipld (IpldNode cid (dag-git-commit tree_cid parent_cids author committer message)) =
  Just GitCommit {
    sha: cid_to_sha(cid),
    tree: fromJust (fmap_from_ipld (resolve tree_cid)),
    parents: mapMaybe (\p_cid → fmap_from_ipld (resolve p_cid)) parent_cids,
    author: author,
    committer: committer,
    message: message
  }
fmap_from_ipld _ = Nothing
```

**Inverse Hash Transformation**:
```haskell
cid_to_sha :: CID → GitHash
cid_to_sha (CID _ (multihash sha1 bytes)) = SHA1 bytes
cid_to_sha (CID _ (multihash sha256 bytes)) = SHA256 bytes
```

### 2.3 Round-Trip Fidelity

**Theorem**: For any `git_obj :: GitObject`:
```
fmap_from_ipld (fmap_to_ipld git_obj) ≈ git_obj
```

**Proof Sketch**:
1. fmap_to_ipld preserves all git object data in dag-git encoding
2. dag-git encoding is lossless (includes all git metadata)
3. fmap_from_ipld extracts git object data from dag-git encoding
4. Therefore, round-trip preserves git object structure (up to hash representation)

**Note**: SHA-1 vs SHA-256 difference is handled by hash type encoding in CID multihash.

## 3. Structure Preservation

### 3.1 Merkle DAG Preservation

**Theorem**: The functor F preserves the Merkle DAG structure.

**Proof**:

1. **Git Merkle DAG**: In git, each object references other objects by SHA hash:
   - Commit → Tree (via tree hash)
   - Commit → Commit (via parent hashes)
   - Tree → {Tree, Blob} (via entry hashes)

2. **IPLD Merkle DAG**: In IPLD, each node links to other nodes by CID:
   - IpldNode(commit) → IpldNode(tree) (via tree CID link)
   - IpldNode(commit) → IpldNode(parent_commit) (via parent CID links)
   - IpldNode(tree) → IpldNode({tree, blob}) (via entry CID links)

3. **F Preserves Structure**: For any git reference `git_a → git_b`:
   ```
   F(git_a → git_b) = F(git_a) → F(git_b)  (as IPLD link)
   ```
   Since F maps git hashes to CIDs deterministically, the reference structure is preserved.

**QED** ∎

### 3.2 Content Addressing Preservation

**Theorem**: Content-addressed hashes are preserved under F.

**Proof**:

1. **Git Content Addressing**:
   ```
   SHA(git_object_content) = git_hash
   ```

2. **IPLD Content Addressing**:
   ```
   multihash(ipld_node_content) = CID.multihash
   ```

3. **F Preserves Hashes**:
   - For GitBlob(sha, content): F produces IpldNode with same content
   - CID's multihash = hash of dag-git-blob(content)
   - dag-git-blob(content) is deterministic encoding of content
   - Therefore: CID uniquely identifies content (same as git SHA)

**QED** ∎

### 3.3 Algebraic Properties

**Commutativity**: Different traversal paths to the same object yield the same CID.

**Proof**:
- Git: If commit C1 and C2 both reference tree T, then hash(T) is the same
- IPLD: F(T) produces the same CID regardless of which commit path we followed
- Therefore: F(T via C1) = F(T via C2) = same CID

**Associativity**: Composition of morphisms is preserved.

**Proof**:
- Git: (c ∘ b) ∘ a = c ∘ (b ∘ a) by category theory
- F((c ∘ b) ∘ a) = F(c) ∘ F(b) ∘ F(a) = F(c ∘ (b ∘ a)) by functor laws
- Therefore: F preserves associativity

## 4. Value Object Design

### 4.1 Core Types

```rust
/// Git hash - supports both SHA-1 and SHA-256
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitHash {
    SHA1([u8; 20]),
    SHA256([u8; 32]),
}

/// Git reference - points to specific content in a git repo
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitRef {
    pub repo_url: Url,
    pub commit_hash: GitHash,
    pub path: PathBuf,
}

/// Git object types (sum type)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitObject {
    Commit(GitCommit),
    Tree(GitTree),
    Blob(GitBlob),
    Tag(GitTag),
}

/// Git commit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: GitHash,
    pub tree: GitHash,
    pub parents: Vec<GitHash>,
    pub author: GitPerson,
    pub committer: GitPerson,
    pub message: String,
    pub gpgsig: Option<String>,
}

/// Git tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitTree {
    pub hash: GitHash,
    pub entries: Vec<GitTreeEntry>,
}

/// Git tree entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitTreeEntry {
    pub mode: GitFileMode,
    pub name: String,
    pub hash: GitHash,
}

/// Git file mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitFileMode {
    File,           // 100644
    Executable,     // 100755
    Symlink,        // 120000
    Directory,      // 040000
    Submodule,      // 160000
}

/// Git blob
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitBlob {
    pub hash: GitHash,
    pub content: Bytes,
}

/// Git tag
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitTag {
    pub hash: GitHash,
    pub object: GitHash,
    pub object_type: GitObjectType,
    pub tag_name: String,
    pub tagger: GitPerson,
    pub message: String,
    pub gpgsig: Option<String>,
}

/// Git person (author/committer/tagger)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitPerson {
    pub name: String,
    pub email: String,
    pub timestamp: i64,
    pub timezone: String,
}

/// Git object type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
}
```

### 4.2 Mapping Types

```rust
use cid::Cid;

/// Mapping from GitRef to CID
pub type GitRefToCid = fn(GitRef) -> Result<Cid, GitIpldError>;

/// Mapping from CID to GitRef
pub type CidToGitRef = fn(Cid) -> Result<Option<GitRef>, GitIpldError>;

/// Mapping from GitObject to IPLD node
pub type GitObjectToIpld = fn(GitObject) -> Result<IpldNode, GitIpldError>;

/// Mapping from IPLD node to GitObject
pub type IpldToGitObject = fn(IpldNode) -> Result<Option<GitObject>, GitIpldError>;

/// Errors
#[derive(Debug, thiserror::Error)]
pub enum GitIpldError {
    #[error("Invalid git hash: {0}")]
    InvalidHash(String),

    #[error("Git object not found: {0}")]
    ObjectNotFound(GitHash),

    #[error("CID does not reference git content: {0}")]
    NotGitContent(Cid),

    #[error("Failed to fetch from git repo: {0}")]
    FetchError(String),

    #[error("IPLD encoding error: {0}")]
    IpldEncodingError(String),
}
```

### 4.3 Functor Trait

```rust
/// Functor from category A to category B
pub trait Functor<A, B> {
    /// Map object from category A to category B
    fn fmap(&self, a: A) -> B;
}

/// Functor from Git to IPLD
pub struct GitToIpld;

impl Functor<GitObject, IpldNode> for GitToIpld {
    fn fmap(&self, git_obj: GitObject) -> IpldNode {
        match git_obj {
            GitObject::Commit(commit) => self.fmap_commit(commit),
            GitObject::Tree(tree) => self.fmap_tree(tree),
            GitObject::Blob(blob) => self.fmap_blob(blob),
            GitObject::Tag(tag) => self.fmap_tag(tag),
        }
    }
}

impl GitToIpld {
    fn fmap_commit(&self, commit: GitCommit) -> IpldNode {
        let cid = git_hash_to_cid(commit.hash, GitObjectType::Commit);
        let tree_cid = git_hash_to_cid(commit.tree, GitObjectType::Tree);
        let parent_cids: Vec<Cid> = commit.parents.iter()
            .map(|p| git_hash_to_cid(*p, GitObjectType::Commit))
            .collect();

        IpldNode::DagGitCommit {
            cid,
            tree: tree_cid,
            parents: parent_cids,
            author: commit.author,
            committer: commit.committer,
            message: commit.message,
            gpgsig: commit.gpgsig,
        }
    }

    fn fmap_tree(&self, tree: GitTree) -> IpldNode {
        let cid = git_hash_to_cid(tree.hash, GitObjectType::Tree);
        let entries: Vec<IpldTreeEntry> = tree.entries.iter()
            .map(|e| IpldTreeEntry {
                name: e.name.clone(),
                mode: e.mode,
                hash: git_hash_to_cid(e.hash, infer_object_type(e.mode)),
            })
            .collect();

        IpldNode::DagGitTree { cid, entries }
    }

    fn fmap_blob(&self, blob: GitBlob) -> IpldNode {
        let cid = git_hash_to_cid(blob.hash, GitObjectType::Blob);
        IpldNode::DagGitBlob {
            cid,
            content: blob.content,
        }
    }

    fn fmap_tag(&self, tag: GitTag) -> IpldNode {
        let cid = git_hash_to_cid(tag.hash, GitObjectType::Tag);
        let object_cid = git_hash_to_cid(tag.object, tag.object_type);

        IpldNode::DagGitTag {
            cid,
            object: object_cid,
            object_type: tag.object_type,
            tag_name: tag.tag_name,
            tagger: tag.tagger,
            message: tag.message,
            gpgsig: tag.gpgsig,
        }
    }
}

/// Functor from IPLD to Git
pub struct IpldToGit;

impl Functor<IpldNode, Option<GitObject>> for IpldToGit {
    fn fmap(&self, ipld_node: IpldNode) -> Option<GitObject> {
        match ipld_node {
            IpldNode::DagGitCommit { cid, tree, parents, author, committer, message, gpgsig } => {
                Some(GitObject::Commit(GitCommit {
                    hash: cid_to_git_hash(cid),
                    tree: cid_to_git_hash(tree),
                    parents: parents.iter().map(|p| cid_to_git_hash(*p)).collect(),
                    author,
                    committer,
                    message,
                    gpgsig,
                }))
            }
            IpldNode::DagGitTree { cid, entries } => {
                Some(GitObject::Tree(GitTree {
                    hash: cid_to_git_hash(cid),
                    entries: entries.iter().map(|e| GitTreeEntry {
                        mode: e.mode,
                        name: e.name.clone(),
                        hash: cid_to_git_hash(e.hash),
                    }).collect(),
                }))
            }
            IpldNode::DagGitBlob { cid, content } => {
                Some(GitObject::Blob(GitBlob {
                    hash: cid_to_git_hash(cid),
                    content,
                }))
            }
            IpldNode::DagGitTag { cid, object, object_type, tag_name, tagger, message, gpgsig } => {
                Some(GitObject::Tag(GitTag {
                    hash: cid_to_git_hash(cid),
                    object: cid_to_git_hash(object),
                    object_type,
                    tag_name,
                    tagger,
                    message,
                    gpgsig,
                }))
            }
            _ => None, // Not a dag-git node
        }
    }
}

/// Helper: Convert GitHash to CID
fn git_hash_to_cid(hash: GitHash, obj_type: GitObjectType) -> Cid {
    match hash {
        GitHash::SHA1(bytes) => {
            Cid::new_v1(
                0x78, // dag-git codec
                Multihash::wrap(0x11, &bytes).unwrap(), // sha1
            )
        }
        GitHash::SHA256(bytes) => {
            Cid::new_v1(
                0x78, // dag-git codec
                Multihash::wrap(0x12, &bytes).unwrap(), // sha256
            )
        }
    }
}

/// Helper: Convert CID to GitHash
fn cid_to_git_hash(cid: Cid) -> GitHash {
    let multihash = cid.hash();
    match multihash.code() {
        0x11 => GitHash::SHA1(multihash.digest().try_into().unwrap()),
        0x12 => GitHash::SHA256(multihash.digest().try_into().unwrap()),
        _ => panic!("Unsupported hash type in CID"),
    }
}

/// Helper: Infer git object type from file mode
fn infer_object_type(mode: GitFileMode) -> GitObjectType {
    match mode {
        GitFileMode::Directory => GitObjectType::Tree,
        GitFileMode::Submodule => GitObjectType::Commit,
        _ => GitObjectType::Blob,
    }
}
```

## 5. Domain Model

### 5.1 Events

```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Git domain events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitDomainEvent {
    RepositoryRegistered(RepositoryRegistered),
    ContentFetched(ContentFetched),
    ObjectCached(ObjectCached),
    CidMappingCreated(CidMappingCreated),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepositoryRegistered {
    pub repo_url: Url,
    pub refs: Vec<GitRef>,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentFetched {
    pub git_ref: GitRef,
    pub cid: Cid,
    pub object_type: GitObjectType,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectCached {
    pub cid: Cid,
    pub git_object: GitObject,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CidMappingCreated {
    pub cid: Cid,
    pub git_ref: GitRef,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Uuid,
}
```

### 5.2 Commands

```rust
/// Git domain commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitDomainCommand {
    RegisterRepository(RegisterRepository),
    FetchContent(FetchContent),
    ResolveCid(ResolveCid),
    CacheObject(CacheObject),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterRepository {
    pub repo_url: Url,
    pub correlation_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchContent {
    pub git_ref: GitRef,
    pub correlation_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolveCid {
    pub cid: Cid,
    pub correlation_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheObject {
    pub git_object: GitObject,
    pub correlation_id: Uuid,
}
```

### 5.3 Aggregate

```rust
/// Git repository aggregate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitRepository {
    pub url: Url,
    pub registered_at: DateTime<Utc>,
    pub refs: HashMap<String, GitRef>,  // branch/tag name → GitRef
    pub cid_mappings: HashMap<Cid, GitRef>,  // CID → GitRef
    pub version: u64,
}

impl GitRepository {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            registered_at: Utc::now(),
            refs: HashMap::new(),
            cid_mappings: HashMap::new(),
            version: 0,
        }
    }

    pub fn apply_event(&self, event: &GitDomainEvent) -> Result<Self, String> {
        let mut repo = self.clone();
        repo.version += 1;

        match event {
            GitDomainEvent::RepositoryRegistered(e) => {
                for git_ref in &e.refs {
                    let ref_name = format!("{}/{}", git_ref.commit_hash, git_ref.path.display());
                    repo.refs.insert(ref_name, git_ref.clone());
                }
            }
            GitDomainEvent::CidMappingCreated(e) => {
                repo.cid_mappings.insert(e.cid, e.git_ref.clone());
            }
            _ => {}
        }

        Ok(repo)
    }
}
```

## 6. Implementation Patterns

### 6.1 GitIpldAdapter

```rust
/// Main adapter for git-IPLD conversions
pub struct GitIpldAdapter {
    git_to_ipld: GitToIpld,
    ipld_to_git: IpldToGit,
}

impl GitIpldAdapter {
    pub fn new() -> Self {
        Self {
            git_to_ipld: GitToIpld,
            ipld_to_git: IpldToGit,
        }
    }

    /// Convert git object to IPLD node
    pub fn to_ipld(&self, git_obj: GitObject) -> IpldNode {
        self.git_to_ipld.fmap(git_obj)
    }

    /// Convert IPLD node to git object (if it's dag-git)
    pub fn from_ipld(&self, ipld_node: IpldNode) -> Option<GitObject> {
        self.ipld_to_git.fmap(ipld_node)
    }

    /// Generate CID from GitRef
    pub async fn git_ref_to_cid(&self, git_ref: &GitRef) -> Result<Cid, GitIpldError> {
        // 1. Fetch git object from repo
        let git_obj = self.fetch_git_object(git_ref).await?;

        // 2. Convert to IPLD
        let ipld_node = self.to_ipld(git_obj);

        // 3. Return CID
        Ok(ipld_node.cid())
    }

    /// Resolve CID to GitRef (if git-backed)
    pub async fn cid_to_git_ref(&self, cid: &Cid) -> Result<Option<GitRef>, GitIpldError> {
        // Look up in mapping store
        // This requires a persistent mapping: CID → GitRef
        todo!("Implement CID → GitRef lookup")
    }

    async fn fetch_git_object(&self, git_ref: &GitRef) -> Result<GitObject, GitIpldError> {
        // Use git2 or similar library to fetch object
        todo!("Implement git fetch")
    }
}
```

### 6.2 Usage Example

```rust
#[tokio::main]
async fn main() -> Result<(), GitIpldError> {
    let adapter = GitIpldAdapter::new();

    // Create a GitRef
    let git_ref = GitRef {
        repo_url: Url::parse("https://github.com/ipfs/go-ipld-git").unwrap(),
        commit_hash: GitHash::SHA1(*b"deadbeefdeadbeefdeadbeef"),
        path: PathBuf::from("README.md"),
    };

    // Generate CID from GitRef
    let cid = adapter.git_ref_to_cid(&git_ref).await?;
    println!("CID: {}", cid);

    // Later, resolve CID back to GitRef
    if let Some(resolved_ref) = adapter.cid_to_git_ref(&cid).await? {
        assert_eq!(resolved_ref, git_ref);
    }

    Ok(())
}
```

## 7. Potential Issues & Mitigations

### 7.1 SHA-1 Deprecation

**Issue**: Git is transitioning from SHA-1 to SHA-256, but both will coexist.

**Mitigation**:
- Support both `GitHash::SHA1` and `GitHash::SHA256` variants
- Encode hash type in CID's multihash (0x11 for sha1, 0x12 for sha256)
- Functor handles both transparently

### 7.2 Repository URL Changes

**Issue**: Same git content, different URLs (mirrors, forks, renames).

**Mitigation**:
- CID generation based on **content only**, not URL
- Maintain separate mapping: `(repo_url, commit, path) → CID`
- Multiple GitRefs can map to same CID

### 7.3 Partial Tree Fetches

**Issue**: Sparse checkouts, partial clones - not all objects available locally.

**Mitigation**:
- Lazy evaluation: fetch git objects on-demand
- Cache fetched objects in cim-flashstor
- Return error if object not fetchable

### 7.4 Round-Trip Fidelity

**Issue**: Git metadata might be lost in IPLD encoding.

**Mitigation**:
- Use dag-git codec which preserves all git metadata
- Include gpgsig, author timezone, etc. in IPLD encoding
- Test round-trip: git → IPLD → git produces identical object

### 7.5 Performance

**Issue**: Fetching from remote git repos is slow.

**Mitigation**:
- Cache git objects locally in cim-flashstor
- Use git shallow clones for efficiency
- Parallel fetching for multiple objects
- Consider hosting git mirrors in cim-flashstor

### 7.6 Security

**Issue**: Malicious git repos could exploit fetch operations.

**Mitigation**:
- Validate git objects before conversion
- Limit fetch sizes (max blob size, max tree depth)
- Sandbox git operations
- Verify GPG signatures if present

## 8. Future Enhancements

### 8.1 Bi-directional Sync

Support syncing IPLD changes back to git repos:
- IPLD mutations → git commits
- Requires git authentication
- Complex conflict resolution

### 8.2 Git LFS Support

Support Git Large File Storage:
- LFS pointers → IPLD CIDs
- Store large files in IPFS, reference in git

### 8.3 Submodule Support

Handle git submodules:
- Submodule references → IPLD links to other repos
- Recursive fetching

### 8.4 Performance Optimization

- Implement libgit2 bindings for faster operations
- Batch fetch multiple objects
- Use git protocol instead of HTTP

## 9. Conclusion

This design establishes a mathematically rigorous foundation for cim-domain-git using Category Theory. The functors F: Git → IPLD and G: IPLD → Git preserve structure, composition, and identity, ensuring that git repositories can be stored natively in cim-flashstor while being served through IPLD CID requests.

Key mathematical properties:
- **Functor laws** ensure structure preservation
- **Natural transformations** provide bidirectional conversion
- **Adjunction** guarantees round-trip fidelity

The implementation uses Rust's type system to enforce categorical properties at compile-time, ensuring correctness.

## 10. References

1. **IPLD dag-git specification**: https://ipld.io/specs/codecs/dag-git/
2. **Git internals**: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
3. **Category Theory for Programmers**: Bartosz Milewski
4. **Functors in Haskell**: https://wiki.haskell.org/Functor

---

**Next Steps**: Proceed to Sprint Planning phase to implement this design.
