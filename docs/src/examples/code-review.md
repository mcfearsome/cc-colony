# Code Review Workflow

Set up an automated code review workflow with specialized agents.

## Overview

This tutorial shows how to create a colony that:
- Has a development agent implementing features
- Has a reviewer agent checking code quality
- Uses task coordination for review workflow
- Integrates with your development process

## Setup

### Step 1: Initialize Colony

```bash
cd /path/to/your/project
colony init
```

Configure 2 agents:
1. **developer** - Main development agent
2. **reviewer** - Code review specialist

### Step 2: Configure Agents

Edit `colony.yml`:

```yaml
name: code-review-workflow

agents:
  - id: developer
    role: Software Developer
    focus: Implement features and fix bugs
    model: claude-sonnet-4-20250514
    worktree: agent/dev
    
  - id: reviewer
    role: Code Reviewer
    focus: Review code for quality, best practices, and potential issues
    model: claude-opus-4-20250514
    worktree: agent/review
    startup_prompt: |
      You are a code reviewer focused on:
      - Code quality and maintainability
      - Best practices and design patterns
      - Potential bugs and edge cases
      - Performance considerations
      - Security vulnerabilities
      
      Review changes thoroughly and provide constructive feedback.
```

### Step 3: Start Colony

```bash
colony start
```

## Usage Workflow

### 1. Developer Implements Feature

In the developer pane:
```
> I need to implement user authentication with JWT tokens
```

The developer agent will:
- Create auth module
- Implement login/register endpoints
- Add JWT token generation
- Write tests

### 2. Create Review Task

```bash
colony state task add "Review authentication implementation" \
  --description "Review JWT auth code for security and best practices"
```

### 3. Reviewer Checks Code

In the reviewer pane, assign the task:
```
> Review the authentication implementation in the auth module
```

The reviewer will:
- Check security vulnerabilities
- Verify input validation
- Review error handling
- Check test coverage
- Suggest improvements

### 4. Address Feedback

Developer implements suggested changes:
```
> Address the security concerns raised by the reviewer:
> 1. Add rate limiting to login endpoint
> 2. Implement token rotation
> 3. Add comprehensive input validation
```

### 5. Final Review

```bash
colony state task update <task-id> completed
```

## Advanced: Automated Workflow

Create a workflow file `.colony/workflows/code-review.yaml`:

```yaml
name: code-review-pipeline
description: Automated code review workflow

steps:
  - id: implement
    agent: developer
    action: implement_feature
    
  - id: review
    agent: reviewer
    action: review_code
    depends_on: [implement]
    
  - id: address-feedback
    agent: developer
    action: address_feedback
    depends_on: [review]
    
  - id: final-check
    agent: reviewer
    action: final_review
    depends_on: [address-feedback]
```

Run the workflow:
```bash
colony workflow run code-review-pipeline
```

## Monitoring

### Watch Progress

```bash
colony tui
```

The TUI shows:
- Agent status
- Task queue
- Active work
- Completed tasks

### View Logs

```bash
# All logs
colony logs

# Developer logs
colony logs developer

# Reviewer logs
colony logs reviewer

# Follow in real-time
colony logs reviewer --follow
```

## Tips

### Use Task Dependencies

Structure work with dependencies:

```bash
# Create feature task
colony state task add "Implement auth" --id task-auth

# Create review task that depends on implementation
colony state task add "Review auth" \
  --blockers task-auth
```

### Template-Based Reviews

Use the built-in code-reviewer template:

```bash
colony template install code-reviewer
```

Update `colony.yml`:
```yaml
agents:
  - id: reviewer
    role: Code Reviewer
    focus: Quality and security review
    template: code-reviewer
```

### Multiple Review Stages

Add specialized reviewers:

```yaml
agents:
  - id: developer
    role: Developer
    
  - id: security-reviewer
    role: Security Auditor
    template: security-auditor
    
  - id: code-reviewer
    role: Code Reviewer
    template: code-reviewer
```

## Real-World Example

### Pull Request Review

```bash
# 1. Developer creates feature branch and implements
colony start
# Developer works in their pane

# 2. Create PR review task
colony state task add "Review PR #123" \
  --description "Review authentication feature PR"

# 3. Reviewer checks the changes
# In reviewer pane: git diff main...feature/auth

# 4. Create follow-up tasks
colony state task add "Fix security issue in auth.rs:45"
colony state task add "Add tests for token expiration"

# 5. Developer addresses issues
# In developer pane: work through tasks

# 6. Final approval
colony state task update pr-123 completed
```

## Benefits

- **Parallel Work**: Developer continues on next feature while reviewer checks previous work
- **Consistent Reviews**: Reviewer follows same process every time
- **Documentation**: All feedback and changes tracked in tasks
- **Isolation**: No git conflicts between dev and review worktrees
- **Persistence**: Colony state persists across sessions

## Next Steps

- [Parallel Development](./parallel-dev.md) - Multiple developers working together
- [Testing Pipeline](./testing-pipeline.md) - Automated testing workflow
- [State Management](../concepts/state.md) - Learn about task coordination
