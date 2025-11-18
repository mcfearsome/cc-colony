# Parallel Development

Run multiple developers working on different features simultaneously.

## Scenario

You need to implement three features in parallel:
1. User authentication system
2. Payment processing
3. Email notification service

## Configuration

```yaml
name: parallel-dev

agents:
  - id: auth-dev
    role: Authentication Developer
    focus: User authentication, JWT tokens, OAuth
    model: claude-sonnet-4-20250514
    worktree: agent/auth
    startup_prompt: |
      Focus on authentication features:
      - Implement secure login/register
      - JWT token management
      - Password hashing with bcrypt
      - Session management

  - id: payments-dev
    role: Payments Developer
    focus: Payment processing, Stripe integration
    model: claude-sonnet-4-20250514
    worktree: agent/payments
    startup_prompt: |
      Focus on payment features:
      - Stripe API integration
      - Webhook handling
      - Payment validation
      - Error handling

  - id: email-dev
    role: Email Developer
    focus: Email notifications, templating
    model: claude-sonnet-4-20250514
    worktree: agent/email
    startup_prompt: |
      Focus on email features:
      - Email template system
      - SMTP configuration
      - Queue management
      - Delivery tracking
```

## Workflow

### 1. Start All Agents

```bash
colony start
```

### 2. Create Feature Tasks

```bash
# Authentication feature
colony state task add "Implement JWT authentication" \
  --description "User login/register with JWT tokens"

# Payments feature
colony state task add "Integrate Stripe payments" \
  --description "Payment processing with Stripe"

# Email feature
colony state task add "Build email notification system" \
  --description "Send transactional emails"
```

### 3. Agents Work Independently

Each agent works in their own worktree without conflicts:

- **auth-dev** works on `agent/auth` branch
- **payments-dev** works on `agent/payments` branch
- **email-dev** works on `agent/email` branch

### 4. Monitor Progress

```bash
colony tui
```

View all three agents working simultaneously:
- Real-time task status
- Agent activity
- Completion progress

### 5. Integration Phase

When features are ready, merge them:

```bash
# Each agent's work is on their branch
git checkout main
git merge agent/auth
git merge agent/payments
git merge agent/email
```

## Advanced: Cross-Agent Dependencies

Some features depend on others:

```yaml
# Payments needs auth
colony state task add "Add payment endpoints" \
  --blockers task-auth-complete

# Email needs both
colony state task add "Send payment confirmation emails" \
  --blockers task-auth-complete,task-payments-complete
```

## Coordination

### Shared State

Agents can communicate via shared state:

```bash
# Auth dev marks API contract ready
colony state memory add context \
  --key "auth_api_contract" \
  --value "POST /api/auth/login, POST /api/auth/register"

# Payments dev can reference it
colony state memory search "auth_api"
```

### Message Passing

```bash
# Send message to specific agent
colony messages send payments-dev \
  "Auth API is ready at /api/auth/*"
```

## Benefits

- **No Git Conflicts**: Each agent has isolated worktree
- **Parallel Execution**: All agents work simultaneously
- **Independent Progress**: Features don't block each other
- **Clear Separation**: Each agent focuses on their domain
- **Easy Integration**: Merge branches when ready

## Real-World Example

### Startup Sprint

```yaml
name: startup-sprint

agents:
  - id: frontend
    role: Frontend Developer
    focus: React, TypeScript, UI components

  - id: backend
    role: Backend Developer
    focus: Node.js, Express, API endpoints

  - id: database
    role: Database Engineer
    focus: PostgreSQL, migrations, queries

  - id: devops
    role: DevOps Engineer
    focus: Docker, deployment, CI/CD
```

All four agents work in parallel:
```bash
colony start
# Frontend builds UI while backend implements API
# Database designs schema while DevOps sets up infrastructure
```

## See Also

- [Code Review Workflow](./code-review.md) - Add review process
- [Testing Pipeline](./testing-pipeline.md) - Automated testing
- [State Management](../concepts/state.md) - Coordinate agents
