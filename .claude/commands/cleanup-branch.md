---
description: "Safely remove a git worktree, local branch, and remote branch with validation"
argument-hint: "[exact-branch-name]"
allowed-tools: ["Bash"]
---

Clean up branch $1 by removing worktree, local branch, and remote branch.

⚠️  **Safety First**: This command will completely remove the branch from all locations.
Make sure you've merged any important work before cleanup.

**Pre-flight checks:**
- Verify the branch exists before attempting cleanup
- Show current state with `git branch -va`
- If branch not found, suggest similar branch names to help with typos

**What this command does:**
1. Shows all branches to confirm target exists
2. Removes the local worktree directory
3. Deletes the local branch reference
4. Removes the branch from remote origin
5. Shows final state to confirm cleanup

**🔍 IMPORTANT NAMING DIFFERENCE:**
- **Branch name**: Uses slashes (e.g., `feat/keyboard-shortcuts`)
- **Worktree directory**: Uses dashes (e.g., `feat-keyboard-shortcuts`)
- When cleaning up `feat/keyboard-shortcuts`, I look for directory `../feat-keyboard-shortcuts`
- If cleanup fails, check both the branch name (with /) and directory name (with -)

**Usage**: `/cleanup-branch exact-branch-name` (must match exactly)

**Commands executed:**
1. `git branch -va` - Show all branches for verification
2. `git worktree remove ../$(echo "$1" | sed 's/\//-/g')` - Remove worktree
3. `git branch -D $1` - Force delete local branch
4. `git push origin --delete $1` - Delete remote branch
5. `git branch -va` - Confirm final state
6. `git worktree list` - Show remaining worktrees

If any step fails, the command stops safely without affecting other branches.
