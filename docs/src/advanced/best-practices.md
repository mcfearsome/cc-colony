# Best Practices

Guidelines for effective multi-agent orchestration with Colony.

## Agent Configuration

### Sizing Your Colony

**Start Small, Scale Up**
```yaml
# Good: Start with 2-3 agents
agents:
  - id: main-dev
  - id: reviewer
  - id: tester

# Avoid: Too many agents initially
# agents: [10+ agents on day one]
```

**Recommended Colony Sizes:**
- **Small Project** (1-2 developers): 2-3 agents
- **Medium Project** (3-5 developers): 4-6 agents
- **Large Project** (6+ developers): 6-10 agents
- **Maximum Practical**: 12-15 agents (beyond this, coordination overhead increases)

### Agent Specialization

**Do: Clear, Focused Roles**
```yaml
agents:
  - id: backend-api
    role: Backend Engineer
    focus: REST API development and database design

  - id: frontend-ui
    role: Frontend Engineer
    focus: React components and user interface

  - id: security-auditor
    role: Security Specialist
    focus: Security review and vulnerability scanning
```

**Don't: Vague, Overlapping Roles**
```yaml
# Avoid this:
agents:
  - id: dev-1
    role: Developer
    focus: Do stuff
```

### Startup Prompts

**Effective Startup Prompts Include:**

1. **Primary Responsibility**
2. **Specific Focus Areas**
3. **Communication Guidelines**
4. **Quality Standards**

```yaml
startup_prompt: |
  You are the Backend API Engineer for this project.

  PRIMARY RESPONSIBILITIES:
  - Design and implement REST API endpoints
  - Manage database schema and migrations
  - Write API documentation

  FOCUS AREAS:
  - RESTful design principles
  - Database optimization
  - API security (auth, validation, rate limiting)

  COMMUNICATION:
  - Coordinate with frontend-ui for API contracts
  - Notify security-auditor before merging
  - Report blockers immediately

  QUALITY STANDARDS:
  - All endpoints must have tests
  - Follow project naming conventions
  - Document all public APIs
```

## Task Management

### Task Design

**Good Task Characteristics:**
- **Atomic**: One clear objective
- **Testable**: Success criteria defined
- **Sized Right**: 2-4 hours of work
- **Well-Scoped**: Clear boundaries

```bash
# Good Task
colony tasks create api-users-endpoint \
  "Implement /api/users endpoint" \
  "Create GET /api/users with pagination, filtering by role, and proper auth. Include unit tests and update API docs." \
  --priority high

# Too Vague
colony tasks create backend-work \
  "Do backend stuff" \
  "Work on backend"
```

### Task Dependencies

**Use Blockers for Sequencing:**
```bash
# Step 1: Database schema
colony state task add "Create users table schema" \
  --description "Design and implement users table with proper indexes"

# Step 2: Depends on schema
colony state task add "Implement user CRUD" \
  --description "Create, Read, Update, Delete operations for users" \
  --blockers "Create users table schema"

# Step 3: Depends on CRUD
colony state task add "Add user authentication" \
  --description "JWT-based authentication for user endpoints" \
  --blockers "Implement user CRUD"
```

### Task Priority Guidelines

| Priority | When to Use | Examples |
|----------|-------------|----------|
| `critical` | Blocking production, security issues | "Fix auth bypass vulnerability", "Restore down service" |
| `high` | Core features, important bugs | "Implement login flow", "Fix data loss bug" |
| `medium` | Standard features, improvements | "Add user profile page", "Optimize query performance" |
| `low` | Nice-to-haves, refactoring | "Update dependencies", "Improve logging" |

## Communication

### Message Types

**Broadcasts** (`colony broadcast` or `b` in TUI):
```bash
# Good uses:
colony broadcast "🚨 Critical: Security patch required in auth module"
colony broadcast "✅ Sprint complete - all tests passing"
colony broadcast "📢 New API contract available in docs/"

# Avoid:
colony broadcast "Working on stuff"  # Too vague
colony broadcast "Hey reviewer-1..."  # Use direct message instead
```

**Direct Messages**:
```bash
# Coordinate specific work
colony messages send backend-dev "API contract ready for /users endpoint"

# Request help
colony messages send security-audit "Please review auth changes in PR #123"

# Report blockers
colony messages send team-lead "Blocked on database access permissions"
```

### Communication Patterns

**1. Pull Request Workflow**
```bash
# Developer finishes work
colony broadcast "PR #123 ready for review: User authentication"

# Reviewer claims
colony messages send developer-1 "Reviewing PR #123, will have feedback in 30min"

# Review complete
colony messages send developer-1 "PR #123 approved with minor suggestions"
```

**2. Blocker Resolution**
```bash
# Agent hits blocker
colony broadcast "🚫 Blocked: Need database credentials for integration tests"

# Coordinator responds
colony messages send blocked-agent "DB creds in 1Password vault 'Dev Credentials'"

# Agent unblocks
colony tasks unblock task-123
colony broadcast "✅ Unblocked, resuming work on integration tests"
```

## Shared State

### When to Enable Shared State

**Enable for:**
- Multi-session work (resume after breaks)
- Cross-session coordination
- Long-running projects
- Distributed teams

**Skip for:**
- Quick experiments
- Single-session work
- Prototype/spike projects

### State Hygiene

**Regular Sync:**
```bash
# Before starting work
colony state pull

# After significant progress
colony state push

# End of session
colony state sync
```

**Clean Completed Tasks:**
```bash
# Weekly cleanup
colony state task list --status completed |
  grep "2024-01" | # Old tasks
  xargs -I {} colony state task delete {}
```

**Commit Message Guidelines:**
```bash
# Good commit messages
git commit -m "state: Add task for user profile feature"
git commit -m "state: Mark authentication tasks as complete"
git commit -m "state: Update blockers for API integration"

# Enable in .git/hooks/prepare-commit-msg
```

## Workflows

### Workflow Design Principles

**1. Single Responsibility**
Each workflow should handle one logical process.

```yaml
# Good: Focused workflow
workflow:
  name: code-review-workflow
  steps:
    - name: lint
    - name: test
    - name: security-scan
    - name: manual-review

# Avoid: Kitchen sink workflow
workflow:
  name: do-everything
  steps: [50+ steps]
```

**2. Idempotent Steps**
Steps should be safely re-runnable.

```yaml
# Good: Can retry
steps:
  - name: run-tests
    agent: test-runner
    retry:
      max_attempts: 3
      backoff: exponential

# Avoid: Non-idempotent
steps:
  - name: increment-counter  # Don't do this
```

**3. Clear Dependencies**
```yaml
steps:
  - name: build
    agent: builder

  - name: test
    depends_on: [build]  # Explicit dependency
    agent: tester

  - name: deploy
    depends_on: [test]  # Sequential
    agent: deployer
```

### Error Handling

**Graceful Degradation:**
```yaml
workflow:
  steps:
    - name: primary-build
      agent: builder
      on_failure: try-backup-build

    - name: try-backup-build
      agent: backup-builder
      on_failure: notify-team

    - name: notify-team
      agent: coordinator
      instructions: "Send alert about build failure"
```

## Monitoring

### What to Monitor

**Critical Metrics:**
1. **Agent Health**: Running vs. failed agents
2. **Task Velocity**: Tasks completed per hour
3. **Message Flow**: Communication patterns
4. **Error Rate**: Failed tasks and retries

### Using the TUI Effectively

**Daily Workflow:**
```bash
# Morning: Check overnight progress
colony tui
# 1. Review Agents tab for any failures
# 2. Check Tasks tab for completion rate
# 3. Scan Messages for any issues

# During Work: Monitor in real-time
# Keep TUI open in dedicated terminal
# Watch for agent failures
# Monitor task queue depth

# Evening: Final check
# 1. Verify all tasks claimed/completed
# 2. Check no blocked agents
# 3. Sync state before shutdown
```

### Log Review

**Regular Log Patterns:**
```bash
# Daily error scan
colony logs --level error --last 24h

# Agent-specific debugging
colony logs problematic-agent --pattern "error|warning"

# Performance monitoring
colony logs --pattern "slow|timeout|retry"
```

## Performance Optimization

### Resource Management

**CPU and Memory:**
- Limit concurrent agents based on system resources
- Use `claude-sonnet` for routine tasks (faster, cheaper)
- Use `claude-opus` only for complex reasoning

```yaml
agents:
  - id: code-reviewer
    model: claude-sonnet-4  # Sufficient for reviews

  - id: architect
    model: claude-opus-4  # Complex system design
```

### Git Worktree Strategy

**Shared vs. Isolated:**

```yaml
# Isolated: Parallel independent work
agents:
  - id: feature-a-dev
    worktree_branch: feature/payment-system

  - id: feature-b-dev
    worktree_branch: feature/notification-service

# Shared: Collaborative work
agents:
  - id: backend-dev
    worktree: shared-api-work
    worktree_branch: feature/api-refactor

  - id: api-tester
    worktree: shared-api-work  # Same worktree
    worktree_branch: feature/api-refactor
```

## Security

### Secrets Management

**Do:**
```yaml
# Use environment variables
agents:
  - id: deploy-agent
    env:
      AWS_ACCESS_KEY_ID: $AWS_ACCESS_KEY_ID  # From environment
      AWS_SECRET_ACCESS_KEY: $AWS_SECRET_ACCESS_KEY
```

**Don't:**
```yaml
# Never hardcode secrets
agents:
  - id: deploy-agent
    startup_prompt: "Use API key: sk-1234abcd..."  # NEVER DO THIS
```

### Access Control

**Principle of Least Privilege:**
```yaml
agents:
  - id: readonly-reviewer
    # Give minimal permissions
    # Can read code, can't push

  - id: deployer
    # Only this agent can deploy
    env:
      DEPLOY_KEY: $DEPLOY_KEY
```

## Team Collaboration

### Multi-User Colonies

**Personal Colonies:**
```yaml
# alice/colony.yml
name: alice-dev-colony
agents:
  - id: alice-main-dev
  - id: alice-reviewer

# bob/colony.yml
name: bob-dev-colony
agents:
  - id: bob-main-dev
  - id: bob-tester
```

**Shared State Coordination:**
```bash
# Alice
colony state pull  # Get latest tasks
colony state task claim task-123 alice-main-dev
colony state push

# Bob
colony state pull  # Sees Alice claimed task-123
colony state task claim task-456 bob-main-dev
colony state push
```

### Code Review Process

**Automated Review Colony:**
```yaml
agents:
  - id: developer
    focus: Implement features

  - id: auto-reviewer
    template: code-reviewer
    startup_prompt: |
      Review all PRs for:
      - Code quality
      - Test coverage
      - Security issues
      - Performance concerns

      Comment inline and request changes if needed.

  - id: security-reviewer
    template: security-auditor
    startup_prompt: |
      Security-focused review:
      - Check for OWASP Top 10
      - Validate input sanitization
      - Review authentication/authorization
```

## Troubleshooting

### Common Issues

**Agents Not Starting:**
```bash
# Check colony health
colony health

# Verify configuration
cat colony.yml

# Check tmux session
tmux list-sessions
colony attach  # See what's happening
```

**Tasks Not Being Claimed:**
```bash
# Check task status
colony tasks list --status pending

# Verify dependencies
colony tasks show task-id

# Check agent assignment
colony tasks agent agent-id
```

**State Sync Conflicts:**
```bash
# Pull latest
colony state pull

# Resolve conflicts manually
# Edit .colony/state/ files

# Push resolved state
colony state push
```

### Debug Mode

**Verbose Logging:**
```yaml
# colony.yml
observability:
  logging:
    level: debug  # Detailed logs
    output: both  # File + stdout
```

## Cost Optimization

### Efficient API Usage

**Model Selection:**
```yaml
# Use cheaper models where appropriate
agents:
  - id: linter
    model: claude-sonnet-4  # Simple tasks

  - id: architect
    model: claude-opus-4  # Complex decisions only
```

**Task Batching:**
```bash
# Instead of many small tasks
colony tasks create review-file-1 "Review file1.js"
colony tasks create review-file-2 "Review file2.js"
# ... (100 tasks)

# Batch similar work
colony tasks create review-batch-1 \
  "Review all files in src/components/" \
  "Check all .js files for code quality issues"
```

### Resource Monitoring

```bash
# Track colony costs
colony metrics show api_calls --hours 24
colony metrics show token_usage --hours 24

# Budget alerts (in your monitoring)
if [ $(colony metrics show token_usage) -gt 1000000 ]; then
  echo "High token usage - review colony efficiency"
fi
```

## Maintenance

### Regular Cleanup

**Weekly:**
```bash
# Clean old tasks
colony tasks list --status completed |
  grep "$(date -d '7 days ago' +%Y-%m)"  |
  xargs -I {} colony tasks delete {}

# Clean old logs
colony logs --clean --older-than 7d
```

**Monthly:**
```bash
# Review agent performance
colony metrics export --output metrics.json
# Analyze which agents are most effective

# Update dependencies
cd ~/.colony/plugins && git pull
colony plugin update --all

# Review and update templates
colony template list
colony template update --all
```

### Backup Strategy

**Critical Data:**
```bash
# Backup colony configuration
cp colony.yml colony.yml.backup

# Backup shared state
git clone .colony/state/.git state-backup

# Backup custom templates
tar -czf templates-backup.tar.gz .colony/templates/
```

## Summary

**Golden Rules:**
1. **Start small** - 2-3 specialized agents
2. **Clear roles** - Each agent knows its job
3. **Communicate** - Use broadcasts and messages effectively
4. **Monitor** - Watch the TUI, review logs
5. **Sync state** - Pull before work, push after
6. **Optimize costs** - Right model for the task
7. **Document** - Keep colony.yml well-commented
8. **Iterate** - Adjust based on what works

**Success Metrics:**
- Agents rarely fail or block
- Tasks flow smoothly through stages
- Communication is clear and purposeful
- State stays synchronized
- Team velocity increases

**When in Doubt:**
- Check the TUI for current state
- Review logs for errors
- Consult this guide
- Ask in community discussions
