---
description: "Create a new git branch with worktree setup for Rust development"
argument-hint: "[branch-name]"
allowed-tools: ["Bash"]
---

Create a new git branch and worktree named: $1

🎯 **Smart Branch Creation**: This command handles both new and existing branches intelligently.

**Before creating, I'll check for:**
- Existing branches with similar names (to catch typos)
- Whether the exact branch already exists locally
- Current branch state with `git branch -va`

**🛑 STOP ON TYPOS**: When potential typos are detected in branch names, STOP and ask for confirmation.
- DO NOT proceed automatically with suspicious names
- Suggest corrections and ask which version to create
- Example: `feat/keyboar-shorcuts` → STOP and ask: "Did you mean `feat/keyboard-shortcuts`?"

**What this command does:**
1. **Pre-check**: Show all branches to detect potential typos
2. **Smart creation**:
   - If branch exists: Create worktree from existing branch
   - If new: Create new branch + worktree
3. **Environment setup**: Copy .env from main branch to new worktree (if exists)
4. **Build check**: Run `cargo check` to verify project compiles
5. **Remote setup**: Push and configure upstream tracking
6. **Verification**: Show final state and usage instructions

**Usage**: `/new-branch branch-name` (I'll help catch typos!)

**Commands executed:**
1. `git branch -va` - Show all branches for typo detection
2. **CRITICAL**: Smart worktree creation with proper directory naming:
   - Check if branch exists: `git show-ref --verify --quiet refs/heads/$1`
   - If existing branch: `git worktree add ../$(echo "$1" | sed 's/\//-/g') $1`
   - If new branch: `git worktree add -b $1 ../$(echo "$1" | sed 's/\//-/g')`
   - **Directory transform**: `feat/keyboard-shortcuts` → `../feat-keyboard-shortcuts`
3. `[ -f .env ] && cp .env ../$(echo "$1" | sed 's/\//-/g')/.env` - Copy environment file if exists
4. `cd ../$(echo "$1" | sed 's/\//-/g') && cargo check && cd -` - Verify project compiles
5. `git push -u origin $1` - Set up remote tracking
6. `git branch -va` - Confirm creation
7. `git worktree list` - Show all active worktrees

**⚠️ IMPORTANT PATH TRANSFORMATION:**
- Branch name: `feat/keyboard-shortcuts`
- Directory path: `../feat-keyboard-shortcuts` (slashes → dashes)
- **NEVER** create directories with slashes in the name!

**Next steps**: Use `cd ../$(echo "$1" | sed 's/\//-/g')` to switch to your new worktree!
