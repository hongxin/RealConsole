---
description: "Rebase current branch onto specified base branch with safety checks"
argument-hint: "[base-branch]"
allowed-tools: ["Bash"]
---

Rebase current branch onto base branch: ${1:-main}

🎯 **Smart Rebase with Safety**: This command performs a safe rebase with comprehensive checks and automatic conflict detection.

**Before rebasing, I'll check for:**
- Clean working directory (no uncommitted changes)
- Current branch status and commit log
- Potential conflicts and divergence from base branch
- Remote synchronization status

**⚠️ SAFETY CHECKS**: This command will STOP and ask for confirmation if:
- Working directory has uncommitted changes
- Current branch is the same as target base branch
- Rebase would result in complex conflicts
- Remote branch is not up-to-date

**What this command does:**
1. **Pre-rebase checks**: Verify clean state and branch relationships
2. **Fetch updates**: Pull latest changes from base branch
3. **Conflict preview**: Show potential conflicts before rebasing
4. **Interactive rebase**: Rebase current branch onto updated base
5. **Force push**: Update remote branch with rebased history
6. **Verification**: Show final state and commit log

**Usage**: `/rebase [base-branch]` (defaults to `main` if not specified)

**Commands executed:**
1. `git status --porcelain` - Check for uncommitted changes
2. `git branch --show-current` - Verify current branch
3. `git log --oneline -10` - Show recent commits before rebase
4. `git fetch origin` - Get latest remote changes
5. `git checkout $BASE && git pull origin $BASE` - Update base branch
6. `git checkout $CURRENT` - Return to current branch
7. `git rebase $BASE` - Perform the rebase
8. `git push --force-with-lease origin $CURRENT` - Update remote safely
9. `git log --oneline -10` - Show commits after rebase
10. `git status` - Final verification

**🔍 CONFLICT HANDLING:**
- If conflicts occur during rebase, the command stops and provides guidance
- Shows conflicted files and suggests resolution steps
- Allows manual resolution before continuing or aborting

**⚠️ IMPORTANT NOTES:**
- This rewrites commit history - use only on feature branches
- Never rebase shared/main branches
- Always ensure important work is backed up
- Force push updates remote branch history

**Next steps**: After successful rebase, your branch is updated and synchronized with the base branch!
