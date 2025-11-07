---
description: "Create a pull request for the current branch with auto-generated description"
argument-hint: "[base-branch (optional)]"
allowed-tools: ["Bash"]
---

Create a pull request for the current branch to ${1:-auto-detected base}.

🎯 **Smart PR Creation**: Automatically detects base branch and generates PR description from git commit history.

**MANDATORY Sanity Checks (performed first):**
1. `pwd` - Show current working directory
2. `git branch --show-current` - Show current branch (this becomes the head branch)
3. `git status` - Check repository state and ensure clean working tree
4. `git remote -v` - Extract repository name for gh CLI
5. Auto-detect base branch from git tracking or common patterns

**Base Branch Selection:**
- **If base branch provided**: Use `$1` as the base branch (e.g., `/create-pr main`)
- **Auto-Detection** (if no argument):
  - First try: `git rev-parse --abbrev-ref @{upstream}` (tracking branch)
  - Fallback: Check if `main` exists, then `master`, then `develop`
  - Show detected base branch and ask for confirmation if uncertain

**Pre-flight Validation:**
- Verify current branch is not the base branch (can't PR to itself)
- Check current branch has commits ahead of base: `git log <base>..<current> --oneline`
- Ensure current branch is pushed to remote: `git ls-remote origin <current>`
- If not pushed, push it: `git push origin <current>`

**What this command does:**
1. **Sanity Checks**: Verify we're in a git repo with a feature branch
2. **Base Branch Detection**: Smart detection of target branch for PR
3. **Repository Detection**: Verify GitHub remote connection
4. **Description Generation**: Create PR description from commit messages between base and current
5. **PR Creation**: Use gh CLI to create the pull request
6. **Output**: Display PR URL and number for future reference

**Usage**:
- `/create-pr` - Auto-detect base branch
- `/create-pr main` - Create PR targeting main branch
- `/create-pr develop` - Create PR targeting develop branch

**Example Flow:**
```bash
# Auto-detect base branch
/create-pr
# Detects base as 'main', creates PR: current → main

# Explicit base branch
/create-pr main
# Creates PR: current → main
```

**Commands executed:**
1. Sanity checks (pwd, current branch, status, remote)
2. Base branch selection (use $1 if provided, otherwise auto-detect)
3. `git log <base>..<current> --oneline` - Get commits for description
4. `git diff <base>..<current> --stat` - Show files changed summary
5. Check if current branch is pushed: `git ls-remote origin <current>`
6. If not pushed: `git push origin <current>`
7. Verify GitHub remote is properly configured
8. Generate structured PR description from commits
9. `gh pr create --head "<current>" --base "<detected-base>" --title "<auto-title>" --body "<auto-description>"`
10. Display created PR details and number

**Auto-generated formats:**
- **Title**: Inferred from branch name and commit messages
- **Description**: Summary + bulleted changes from git log + testing checklist
- **Branch patterns**: `feat/login` → "feat: implement user login functionality"

**🛑 STOP CONDITIONS**:
- If current branch is the base branch
- If no commits ahead of base branch
- If unable to detect base branch
- If gh CLI fails or not authenticated

**Next steps**: Use `/merge-pr <pr-number>` to merge when ready
