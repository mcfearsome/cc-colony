---
name: gh-cli
description: GitHub CLI (gh) for Pull Requests, Issues, and repository operations
---

# GitHub CLI (gh) for Colony Agents

## Purpose

Use the GitHub CLI (`gh`) for creating PRs, managing issues, reviewing code, and automating GitHub workflows without leaving the terminal.

## Prerequisites

```bash
# Check if gh is installed
which gh

# Check auth status
gh auth status

# Login if needed (interactive)
gh auth login
```

## Pull Requests

### Create PR

```bash
# Create PR from current branch
gh pr create

# Create with title and body
gh pr create --title "Add user authentication" --body "Implements JWT-based auth system"

# Create as draft
gh pr create --draft --title "WIP: Refactor components"

# Create with reviewers
gh pr create --title "Fix bug" --reviewer @octocat,@teammate

# Create and assign
gh pr create --title "Feature" --assignee @me

# Set base branch
gh pr create --base develop --head feature/my-feature
```

### List PRs

```bash
# List open PRs
gh pr list

# List your PRs
gh pr list --author @me

# List PRs by state
gh pr list --state closed
gh pr list --state merged
gh pr list --state all

# Filter by label
gh pr list --label bug
gh pr list --label "needs review"
```

### View PR Details

```bash
# View PR #123
gh pr view 123

# View in browser
gh pr view 123 --web

# View diff
gh pr diff 123

# View checks status
gh pr checks 123
```

### Review PRs

```bash
# Check out PR locally
gh pr checkout 123

# Comment on PR
gh pr comment 123 --body "LGTM! :+1:"

# Review PR
gh pr review 123 --approve
gh pr review 123 --request-changes --body "Please fix the type errors"
gh pr review 123 --comment --body "Looks good overall, minor suggestions"
```

### Merge PRs

```bash
# Merge PR
gh pr merge 123

# Squash merge
gh pr merge 123 --squash

# Rebase merge
gh pr merge 123 --rebase

# Delete branch after merge
gh pr merge 123 --delete-branch
```

### PR Status

```bash
# Check CI status
gh pr checks

# Watch checks in real-time
gh pr checks --watch

# View PR status
gh pr status
```

## Issues

### Create Issue

```bash
# Interactive creation
gh issue create

# With title and body
gh issue create --title "Bug: Login fails" --body "Steps to reproduce:
1. Go to /login
2. Enter credentials
3. Error appears"

# With labels and assignees
gh issue create --title "Feature request" --label enhancement --assignee @me
```

### List Issues

```bash
# Open issues
gh issue list

# Your issues
gh issue list --assignee @me

# By label
gh issue list --label bug

# Search
gh issue list --search "authentication"
```

### View and Comment

```bash
# View issue
gh issue view 456

# View in browser
gh issue view 456 --web

# Comment
gh issue comment 456 --body "Working on this now"
```

### Close/Reopen

```bash
# Close issue
gh issue close 456

# Reopen
gh issue reopen 456

# Close with comment
gh issue close 456 --comment "Fixed in PR #123"
```

## Repository Operations

### Clone

```bash
# Clone repo
gh repo clone owner/repo

# Clone your fork
gh repo fork owner/repo --clone
```

### View Repo Info

```bash
# View current repo
gh repo view

# View specific repo
gh repo view owner/repo

# View in browser
gh repo view --web
```

### Create Repo

```bash
# Create new repo
gh repo create my-new-repo --public

# Create private repo
gh repo create my-new-repo --private

# Create with description
gh repo create my-new-repo --description "My awesome project"
```

## Workflows (GitHub Actions)

### List Workflows

```bash
# List all workflows
gh workflow list

# View workflow runs
gh run list

# View specific workflow runs
gh run list --workflow=ci.yml
```

### Trigger Workflow

```bash
# Trigger workflow manually
gh workflow run ci.yml

# With inputs
gh workflow run deploy.yml --field environment=production
```

### View Run Details

```bash
# View run
gh run view 123456

# View logs
gh run view 123456 --log

# Watch run in real-time
gh run watch 123456
```

## Code Search

```bash
# Search code in repo
gh search code "function authenticate" --repo owner/repo

# Search across GitHub
gh search code "colony orchestration" --language rust

# Search with filters
gh search code "JWT validation" --language typescript --path src/
```

## Releases

### Create Release

```bash
# Create release
gh release create v1.0.0 --title "Version 1.0.0" --notes "Initial release"

# Create with assets
gh release create v1.0.0 --notes "Release notes" ./dist/*

# Create draft
gh release create v1.0.0 --draft
```

### List and View

```bash
# List releases
gh release list

# View latest release
gh release view --web

# Download release assets
gh release download v1.0.0
```

## Colony Integration Patterns

### Automated PR Creation

```bash
#!/bin/bash
# Agent creates PR after completing work

BRANCH=$(git branch --show-current)

# Ensure changes are committed
if [[ -n $(git status --porcelain) ]]; then
    echo "Error: Uncommitted changes. Commit first."
    exit 1
fi

# Push branch
git push -u origin "$BRANCH"

# Create PR
PR_URL=$(gh pr create \
    --title "feat: implement feature X" \
    --body "Completed by agent $(whoami)

Changes:
- Added feature X
- Updated tests
- Updated documentation" \
    --label "agent-created" \
    | grep -o 'https://.*')

# Notify other agents
./colony_message.sh send orchestrator "Created PR: $PR_URL"
```

### Check PR Status Before Merging

```bash
#!/bin/bash
# Check if PR is ready to merge

PR_NUMBER=$1

# Check if all checks passed
STATUS=$(gh pr checks "$PR_NUMBER" --json state --jq '.[].state' | sort -u)

if [[ "$STATUS" == "SUCCESS" ]]; then
    echo "All checks passed"
    ./colony_message.sh send orchestrator "PR #$PR_NUMBER ready to merge - all checks passed"
else
    echo "Checks not passing: $STATUS"
    ./colony_message.sh send orchestrator "PR #$PR_NUMBER not ready - failing checks"
fi
```

### Issue Triage

```bash
#!/bin/bash
# Agent monitors and triages new issues

gh issue list --label "needs-triage" --json number,title | jq -r '.[] | "\(.number): \(.title)"' | \
while IFS=: read -r number title; do
    echo "Triaging issue #$number"

    # Simple triage logic
    if echo "$title" | grep -qi "bug\|error\|broken"; then
        gh issue edit "$number" --add-label "bug"
        ./colony_message.sh send orchestrator "Triaged #$number as bug"
    elif echo "$title" | grep -qi "feature\|add\|implement"; then
        gh issue edit "$number" --add-label "enhancement"
        ./colony_message.sh send orchestrator "Triaged #$number as enhancement"
    fi

    gh issue edit "$number" --remove-label "needs-triage"
done
```

## Advanced Usage

### JSON Output for Scripting

```bash
# Get PR data as JSON
gh pr list --json number,title,author,state

# Process with jq
gh pr list --json number,title,state | \
  jq -r '.[] | select(.state=="OPEN") | "#\(.number): \(.title)"'

# Get specific fields
gh pr view 123 --json title,body,author --jq '.author.login'
```

### Custom Queries

```bash
# GraphQL query
gh api graphql -f query='
query {
  repository(owner: "owner", name: "repo") {
    pullRequests(first: 10, states: OPEN) {
      nodes {
        number
        title
        author {
          login
        }
      }
    }
  }
}'
```

### Bulk Operations

```bash
# Close all stale PRs
gh pr list --label stale --json number --jq '.[].number' | \
while read -r pr; do
    gh pr close "$pr" --comment "Closing due to inactivity"
done

# Add label to multiple issues
gh issue list --label bug --json number --jq '.[].number' | \
while read -r issue; do
    gh issue edit "$issue" --add-label "priority:high"
done
```

## Configuration

### Set Defaults

```bash
# Set default repo
gh repo set-default

# Configure editor
gh config set editor nvim

# Set default protocol
gh config set git_protocol ssh
```

### Aliases

```bash
# Create alias
gh alias set prs 'pr list --author @me'

# Use alias
gh prs

# List aliases
gh alias list
```

## Tips

**1. Use `--json` for scripting**
- Parseable output
- Combine with `jq` for processing

**2. Check before acting**
- Use `gh pr view` before merging
- Verify `gh pr checks` pass

**3. Automate repetitive tasks**
- Create aliases for common operations
- Script bulk operations

**4. Communicate via Colony**
- Send PR URLs to other agents
- Notify about important changes
- Request reviews via messages

**5. Use templates**
- `gh pr create --template` for consistency
- Define PR/issue templates in `.github/`

## Common Workflows

### Feature Development

```bash
# 1. Create feature branch
git checkout -b feature/awesome

# 2. Make changes and commit
git add .
git commit -m "feat: add awesome feature"

# 3. Push and create PR
git push -u origin feature/awesome
gh pr create --fill  # Uses commit messages

# 4. Request review
gh pr edit --add-reviewer @teammate

# 5. Monitor checks
gh pr checks --watch

# 6. Merge when ready
gh pr merge --squash --delete-branch
```

### Bug Fix

```bash
# 1. Create from issue
gh issue develop 789 --checkout

# 2. Fix and test
# ... make changes ...

# 3. Commit and push
git commit -am "fix: resolve issue #789"
git push -u origin HEAD

# 4. Create PR that closes issue
gh pr create --title "Fix: issue #789" --body "Closes #789"

# 5. Merge
gh pr merge --squash
```

## Error Handling

```bash
# Check if gh command succeeded
if gh pr create --title "Test"; then
    echo "PR created successfully"
else
    echo "Failed to create PR: $?"
    ./colony_message.sh send orchestrator "Failed to create PR"
    exit 1
fi

# Capture output and errors
output=$(gh pr list 2>&1) || {
    echo "gh command failed"
    exit 1
}
```

## Resources

- Official docs: `gh help`
- PR operations: `gh pr --help`
- Issue operations: `gh issue --help`
- Manual: https://cli.github.com/manual/
