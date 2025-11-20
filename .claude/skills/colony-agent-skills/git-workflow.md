---
name: git-workflow
description: Git workflow and best practices for Colony agents
---

# Git Workflow for Colony Agents

## Purpose

Manage git operations effectively within Colony's multi-agent environment, including branch management, commits, and coordination.

## Your Git Context

### Check Your Current State

```bash
# Current branch
git branch --show-current

# Working directory status
git status --short

# Recent commits
git log --oneline -5

# Your worktree info
git worktree list
```

### Colony Branch Naming

Your worktree uses: `colony/<agent-id>` branch

```bash
# Your branch
git branch --show-current
# Output: colony/frontend-1
```

## Branch Operations

### Create Feature Branch

```bash
# From your current commit
git checkout -b feature/my-feature

# Push and set upstream
git push -u origin feature/my-feature
```

### Switch Branches (Safely)

```bash
# Stash current changes first
git stash push -m "WIP: description"

# Switch branch
git checkout main

# Or create and switch
git checkout -b new-branch

# Restore changes
git stash pop
```

### Branch Information

```bash
# List all branches
git branch -a

# Show branch with last commit
git branch -v

# Check if branch exists
git rev-parse --verify feature/name &>/dev/null && echo "exists"
```

## Committing Changes

### Check What Changed

```bash
# Show changed files
git status --short

# Show changes (unstaged)
git diff

# Show staged changes
git diff --cached

# Show changes in specific file
git diff path/to/file.ts
```

### Stage Changes

```bash
# Stage specific file
git add path/to/file.ts

# Stage all TypeScript files
git add '*.ts'

# Stage interactively (choose hunks)
git add -p file.ts

# Stage all changes
git add -A
```

### Commit

```bash
# Simple commit
git commit -m "feat: add user authentication"

# Commit with description
git commit -m "feat: add user authentication" -m "
- Implement JWT token validation
- Add login/logout endpoints
- Include password hashing with bcrypt
"

# Amend last commit (if not pushed)
git commit --amend -m "Updated message"

# Amend without changing message
git commit --amend --no-edit
```

### Commit Message Format

```
<type>: <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `docs`: Documentation
- `test`: Tests
- `chore`: Maintenance

**Examples:**
```bash
git commit -m "feat: add dark mode toggle"
git commit -m "fix: resolve login redirect loop"
git commit -m "refactor: extract validation to utils"
```

## Synchronizing

### Pull Latest Changes

```bash
# Pull with rebase (cleaner history)
git pull --rebase origin main

# Pull with merge
git pull origin main

# Fetch without merging
git fetch origin
```

### Push Changes

```bash
# Push current branch
git push

# Push and set upstream
git push -u origin $(git branch --show-current)

# Force push (use cautiously!)
git push --force-with-lease origin feature/branch
```

## Handling Conflicts

### When Merge Conflicts Occur

```bash
# See conflicted files
git status

# View conflict in file
cat conflicted-file.ts
# Look for: <<<<<<< HEAD ... ======= ... >>>>>>> branch

# Resolve with editor
nvim conflicted-file.ts
# Remove conflict markers, keep desired code

# Mark as resolved
git add conflicted-file.ts

# Continue operation
git rebase --continue  # if rebasing
git merge --continue   # if merging
```

### Abort if Stuck

```bash
# Abort rebase
git rebase --abort

# Abort merge
git merge --abort

# Abort cherry-pick
git cherry-pick --abort
```

## Useful Git Commands

### History and Log

```bash
# Pretty log with graph
git log --oneline --graph --decorate -10

# Show file history
git log --follow -- path/to/file.ts

# Show who changed what
git blame -L 10,20 file.ts

# Find when bug was introduced
git bisect start
git bisect bad  # Current commit has bug
git bisect good abc123  # This commit was good
# Git will check out commits to test
```

### Stashing

```bash
# Save current work
git stash push -m "description"

# List stashes
git stash list

# Apply and keep stash
git stash apply stash@{0}

# Apply and remove stash
git stash pop

# Show stash contents
git stash show -p stash@{0}

# Drop stash
git stash drop stash@{0}
```

### Undoing Changes

```bash
# Discard unstaged changes in file
git checkout -- file.ts

# Discard all unstaged changes
git checkout -- .

# Unstage file (keep changes)
git reset HEAD file.ts

# Undo last commit (keep changes)
git reset --soft HEAD~1

# Undo last commit (discard changes)
git reset --hard HEAD~1

# Revert a commit (creates new commit)
git revert abc123
```

### Cherry-picking

```bash
# Apply commit from another branch
git cherry-pick abc123

# Cherry-pick range
git cherry-pick abc123..def456

# Cherry-pick without committing
git cherry-pick -n abc123
```

## Colony-Specific Workflows

### Coordinate Branch Work

```bash
#!/bin/bash
# Before starting work on shared code

# Check what others are working on
./colony_message.sh read | grep -i "working on"

# Announce your branch
BRANCH=$(git branch --show-current)
./colony_message.sh send all "Working on $BRANCH: implementing auth system"

# After pushing
git push -u origin "$BRANCH"
./colony_message.sh send all "Pushed $BRANCH - ready for review"
```

### Share Commit References

```bash
# Get current commit SHA
COMMIT=$(git rev-parse HEAD)

# Share with other agents
./colony_message.sh send backend-1 "Review commit $COMMIT for API changes"

# Other agent can check it out
git show "$COMMIT"
```

### Collaborative Fixes

```bash
# Agent 1 creates fix
git checkout -b fix/bug-123
# ... make changes ...
git commit -m "fix: resolve bug-123"
git push -u origin fix/bug-123

./colony_message.sh send frontend-2 "Created fix/bug-123 branch - please test"

# Agent 2 tests
git fetch origin
git checkout fix/bug-123
# ... test ...
./colony_message.sh send frontend-1 "Tested fix/bug-123 - looks good!"
```

## Safety Rules

**DO:**
- ✅ Always check status before committing
- ✅ Write clear commit messages
- ✅ Test before pushing
- ✅ Communicate branch work to other agents
- ✅ Use feature branches for new work

**DON'T:**
- ❌ Force push to shared branches
- ❌ Commit directly to main/master
- ❌ Commit secrets or credentials
- ❌ Make massive commits (break into logical chunks)
- ❌ Forget to pull before starting work

## Troubleshooting

### Accidentally Committed to Wrong Branch

```bash
# Move commit to correct branch
git checkout correct-branch
git cherry-pick wrong-branch
git checkout wrong-branch
git reset --hard HEAD~1
```

### Large Files

```bash
# Check repo size
git count-objects -vH

# Find large files
git rev-list --objects --all | \
  git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | \
  awk '/^blob/ {print substr($0,6)}' | sort -n -k2 | tail -10
```

### Clean Up

```bash
# Remove untracked files
git clean -fd

# Show what would be removed
git clean -fdn

# Prune old branches
git fetch --prune origin

# Delete merged local branches
git branch --merged | grep -v "\*" | xargs -n 1 git branch -d
```
