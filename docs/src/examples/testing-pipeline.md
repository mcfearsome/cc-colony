# Testing Pipeline

Automated test generation and QA workflow.

## Overview

Set up a comprehensive testing pipeline with specialized test agents.

## Configuration

```yaml
name: testing-pipeline

agents:
  - id: developer
    role: Software Developer
    focus: Feature implementation
    model: claude-sonnet-4-20250514
    worktree: agent/dev

  - id: unit-tester
    role: Unit Test Engineer
    template: test-engineer
    model: claude-sonnet-4-20250514
    worktree: agent/unit-tests

  - id: integration-tester
    role: Integration Test Engineer
    template: test-engineer
    model: claude-sonnet-4-20250514
    worktree: agent/integration-tests
```

## Workflow

### 1. TDD Approach

```bash
# Create test task first
colony state task add "Write login tests"

# Feature task depends on tests
colony state task add "Implement login" --blockers test-login
```

### 2. Run Tests

```bash
# Unit tests
npm test -- --coverage

# Integration tests
npm run test:integration
```

## Test Types

### Unit Tests
- Test individual functions
- Mock dependencies
- Aim for 90%+ coverage

### Integration Tests
- Test API endpoints
- Database interactions
- Third-party integrations

### E2E Tests
- Complete user journeys
- Browser automation
- Real environment testing

## Metrics

Track test coverage:

```bash
colony metrics record test.coverage 87.5
colony metrics show test.coverage
```

## Benefits

- High test coverage
- Fast feedback
- Regression prevention
- CI/CD ready

## See Also

- [Workflows](../concepts/workflows.md)
- [Metrics](../cli/metrics.md)
- [Templates](../concepts/templates.md)
