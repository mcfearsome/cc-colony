# Documentation Team

Automatically maintain documentation as code changes.

## Concept

Keep documentation synchronized with code by having dedicated documentation agents monitor changes and update docs.

## Configuration

```yaml
name: docs-team

agents:
  - id: developer
    role: Software Developer
    focus: Feature implementation
    model: claude-sonnet-4-20250514
    worktree: agent/dev

  - id: docs-writer
    role: Documentation Writer
    focus: API docs, user guides, README updates
    template: documentation-writer
    model: claude-sonnet-4-20250514
    worktree: agent/docs
    startup_prompt: |
      Monitor code changes and keep documentation updated:
      - API documentation
      - README files
      - User guides
      - Code comments
      - Architecture docs
```

## Workflow

### 1. Developer Implements Feature

Developer adds new API endpoint:
```typescript
// src/api/users.ts
export async function createUser(data: CreateUserDTO): Promise<User> {
  // Implementation
}
```

### 2. Notify Docs Writer

```bash
colony state task add "Document new user creation API" \
  --description "New POST /api/users endpoint added"
```

### 3. Docs Writer Updates Documentation

Docs writer agent:
- Reads the new code
- Updates API documentation
- Adds usage examples
- Updates CHANGELOG

Creates:
```markdown
# API Documentation

## Create User

**Endpoint**: `POST /api/users`

**Request Body**:
```json
{
  "email": "user@example.com",
  "name": "John Doe"
}
```

**Response**:
```json
{
  "id": "user_123",
  "email": "user@example.com",
  "name": "John Doe",
  "createdAt": "2024-01-15T10:30:00Z"
}
```

**Example**:
```typescript
const user = await createUser({
  email: "user@example.com",
  name: "John Doe"
});
```
```

## Automated Documentation

### Watch for Changes

```bash
# Set up a workflow
cat > .colony/workflows/auto-docs.yaml <<EOF
name: auto-documentation
description: Automatically update docs when code changes

trigger:
  type: file_change
  pattern: "src/**/*.ts"

steps:
  - id: detect-changes
    agent: developer
    action: list_changes

  - id: update-docs
    agent: docs-writer
    action: update_documentation
    depends_on: [detect-changes]
