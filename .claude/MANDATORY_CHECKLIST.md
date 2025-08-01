# MANDATORY CHECKLIST - MUST FOLLOW EVERY TIME

## 🚨 STOP! READ THIS FIRST! 🚨

### BEFORE ANY ACTION, I MUST:

#### 1. Establish Context (MANDATORY)
- [ ] **Infrastructure Location**: Am I in REGISTRY / MODULE / DOMAIN?
  ```bash
  # Optional: Check infrastructure type
  ./.claude/scripts/detect-context.sh
  ```
- [ ] **Bounded Context**: What domain am I working in?
  - Planning → Focus on design, architecture, event storming
  - Coding → TDD in Rust on NixOS, event-driven patterns
  - Testing → Coverage, verification, integration tests
  - Debugging → Compilation errors, test failures
  - Documentation → User guides, API docs, diagrams
- [ ] **Current Task**: What specifically am I being asked to do?

#### 2. Read Core Instructions
- [ ] I have read `.claude/CLAUDE.md` IN THIS SESSION
- [ ] I have checked `.claude/instructions/main-directives.md` for:
  - [ ] Filename conventions (NEVER UPPERCASE)
  - [ ] Date handling (NEVER generate, ALWAYS use system)
  - [ ] No proactive documentation creation

#### 3. Check Current State
```bash
# Read progress to understand current state
cat /git/thecowboyai/cim/doc/progress/progress.json
```
- [ ] I know what phase we're in (DESIGNED/PLANNED/IMPLEMENTED/etc)
- [ ] I know what's blocked
- [ ] I know the Definition of Done

#### 4. Before Creating ANY File
- [ ] Is this file ABSOLUTELY NECESSARY?
- [ ] Have I checked if it already exists?
- [ ] Is the filename lowercase_with_underscores?
- [ ] Am I creating documentation without being asked? (DON'T!)

#### 5. Before Writing Code
- [ ] Have I checked for existing modules to ASSEMBLE?
```bash
./scripts/query-modules.sh --feature <what-i-need>
```
- [ ] Am I following event-driven patterns (NO CRUD)?
- [ ] Am I using state machines where appropriate?

#### 6. Before Claiming Completion
- [ ] Does it compile? `cargo build`
- [ ] Do tests pass? `cargo test`
- [ ] Have I verified each phase of Definition of Done?
  - DESIGNED → Design docs exist
  - PLANNED → Implementation plan exists
  - IMPLEMENTED → Code written
  - VERIFIED → Code compiles
  - TESTED → Tests pass
  - COMPLETE → All features work
  - DONE → Production ready

#### 7. Progress Update Rules
- [ ] Use system date: `CURRENT_DATE=$(date -I)`
- [ ] Accurate status (not wishful thinking)
- [ ] List blockers if any
- [ ] Update completion percentage realistically

#### 8. Before EVERY Commit in cim-network Module
- [ ] **UPDATE PROGRESS.JSON FIRST**
  - [ ] Update completion percentage
  - [ ] Update metrics (files_created, tests_written, tests_passing)
  - [ ] Add new artifacts to the list
  - [ ] Update next_steps
- [ ] **Run tests**: `cargo test`
- [ ] **Commit order**:
  ```bash
  # 1. Update progress FIRST
  cd /git/thecowboyai/cim
  # Edit doc/progress/progress.json
  
  # 2. Commit to cim-network submodule
  cd modules/cim-network
  git add -A
  git commit -m "feat: ..."
  git push origin main
  
  # 3. Commit to parent WITH progress.json
  cd ../..
  git add modules/cim-network doc/progress/progress.json
  git commit -m "feat: Progress on cim-network..."
  git push origin main
  ```

## 📚 CONTEXT CLARIFICATION

**Infrastructure Type** (client/leaf/cluster) ≠ **Bounded Context** (domain focus)

- **Infrastructure Type**: Where code runs (detected by script)
- **Bounded Context**: What domain/problem space I'm working in
  - Network domain → Focus on routing, switching, IP management
  - Storage domain → Focus on persistence, caching, retrieval
  - Identity domain → Focus on authentication, authorization
  - Current Activity → Planning vs Coding vs Testing vs Debugging

**The context that matters is the Bounded Context - what should I be paying attention to?**

## 🛑 VIOLATIONS I KEEP MAKING 🛑

1. **Creating UPPERCASE filenames** → ALWAYS lowercase_with_underscores
2. **Creating docs proactively** → ONLY when explicitly asked
3. **Not checking compilation** → ALWAYS verify before claiming done
4. **Not using existing modules** → ALWAYS check what exists first
5. **Marking incomplete work as DONE** → Be honest about status

## 📋 Quick Reference Commands

```bash
# Context check
./.claude/scripts/detect-context.sh

# Find existing modules
./scripts/query-modules.sh

# Check compilation
cargo build

# Run tests
cargo test

# Get system date
CURRENT_DATE=$(date -I)

# Check what files I've created
git status
```

## ⚠️ FINAL REMINDER ⚠️

**I MUST READ THIS CHECKLIST AT THE START OF EVERY INTERACTION**

If I skip this checklist, I WILL make the same mistakes again. There is no exception to this rule.