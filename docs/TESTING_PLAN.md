# CC-Colony Testing Plan

## Overview

This document outlines a comprehensive testing strategy for cc-colony, a multi-agent orchestration tool for Claude Code instances on tmux. The goal is to achieve robust test coverage across unit, integration, end-to-end, performance, and security testing dimensions.

**Current State**: Minimal test coverage (only 2 config validation tests)
**Target State**: Comprehensive test suite with >80% code coverage and confidence in all critical paths

---

## Table of Contents

1. [Testing Philosophy](#testing-philosophy)
2. [Test Categories](#test-categories)
3. [Unit Tests](#unit-tests)
4. [Integration Tests](#integration-tests)
5. [End-to-End Tests](#end-to-end-tests)
6. [Performance Tests](#performance-tests)
7. [Security Tests](#security-tests)
8. [Test Infrastructure](#test-infrastructure)
9. [CI/CD Integration](#cicd-integration)
10. [Testing Tools & Dependencies](#testing-tools--dependencies)
11. [Test Execution Strategy](#test-execution-strategy)
12. [Coverage Goals](#coverage-goals)
13. [Known Gaps & Future Work](#known-gaps--future-work)

---

## Testing Philosophy

### Principles

1. **Fast Feedback**: Unit tests should run in <5s, integration tests in <30s
2. **Isolation**: Each test should be independent and reproducible
3. **Determinism**: Tests must not be flaky; external dependencies should be mocked
4. **Clarity**: Test names should describe the scenario and expected outcome
5. **Maintainability**: Avoid test duplication; use helper functions and fixtures
6. **Real-World Scenarios**: E2E tests should mirror actual user workflows

### Test Pyramid

```
        /\
       /  \  E2E Tests (10%)
      /    \  - Full CLI workflows
     /------\  - Multi-agent scenarios
    /        \ Integration Tests (30%)
   /          \ - Module interactions
  /            \ - tmux/git integration
 /--------------\ Unit Tests (60%)
                  - Business logic
                  - Data structures
                  - Error handling
```

---

## Test Categories

### 1. Unit Tests (60% of test suite)

**Focus**: Individual functions, structs, and isolated logic
**Location**: `src/**/*.rs` (inline `#[cfg(test)]` modules)
**Dependencies**: Minimal; use mocks for external calls

### 2. Integration Tests (30% of test suite)

**Focus**: Module interactions, file I/O, state management
**Location**: `tests/` directory (Rust convention)
**Dependencies**: Real file system, mocked tmux/git where needed

### 3. End-to-End Tests (10% of test suite)

**Focus**: Full CLI command workflows, multi-agent orchestration
**Location**: `tests/e2e/` subdirectory
**Dependencies**: Real tmux, real git, isolated test environments

### 4. Performance Tests

**Focus**: Stress testing, scalability, resource usage
**Location**: `tests/performance/` or `benches/` (using Criterion)
**Dependencies**: Benchmark harness, system monitoring

### 5. Security Tests

**Focus**: Input validation, path traversal, shell injection
**Location**: `tests/security/` or integrated into unit tests
**Dependencies**: Fuzzing tools (cargo-fuzz), security audit tools

---

## Unit Tests

### Module: `src/colony/config.rs`

**Status**: ✅ Partial coverage (2 tests exist)

#### Existing Tests
- ✅ `test_default_config()` - Validates default configuration
- ✅ `test_duplicate_ids()` - Rejects duplicate agent IDs

#### Additional Tests Needed
- [ ] `test_parse_valid_yaml()` - Parse valid colony.yml
- [ ] `test_parse_invalid_yaml()` - Reject malformed YAML
- [ ] `test_missing_required_fields()` - Error on missing required fields
- [ ] `test_invalid_agent_id_characters()` - Reject special chars in IDs (e.g., `../`, spaces)
- [ ] `test_custom_directory_validation()` - Validate custom directory paths
- [ ] `test_model_name_validation()` - Ensure valid model names
- [ ] `test_empty_agents_list()` - Handle empty agents array
- [ ] `test_large_config()` - Parse config with 50+ agents
- [ ] `test_default_values()` - Verify default model and directory values
- [ ] `test_yaml_edge_cases()` - Handle comments, anchors, multi-line strings

---

### Module: `src/colony/agent.rs`

**Status**: ❌ No tests

#### Tests Needed
- [ ] `test_agent_creation()` - Create agent with valid parameters
- [ ] `test_agent_status_transitions()` - Idle → Running → Completed
- [ ] `test_agent_status_to_failed()` - Transition to Failed state
- [ ] `test_invalid_status_transition()` - Ensure invalid transitions are prevented (if applicable)
- [ ] `test_agent_serialization()` - Serialize/deserialize Agent struct
- [ ] `test_agent_equality()` - Test PartialEq implementation
- [ ] `test_agent_display_formatting()` - Verify Display trait output
- [ ] `test_agent_clone()` - Ensure deep copy works correctly

---

### Module: `src/colony/controller.rs`

**Status**: ❌ No tests

#### Tests Needed
- [ ] `test_controller_initialization()` - Create controller with valid config
- [ ] `test_load_persisted_state()` - Load state from existing state.json
- [ ] `test_save_state()` - Persist state to filesystem
- [ ] `test_add_agent()` - Add agent to controller
- [ ] `test_remove_agent()` - Remove agent by ID
- [ ] `test_update_agent_status()` - Modify agent status
- [ ] `test_get_agent_by_id()` - Retrieve specific agent
- [ ] `test_list_all_agents()` - Get all agents in colony
- [ ] `test_state_file_corruption()` - Handle corrupted state.json gracefully
- [ ] `test_concurrent_state_updates()` - Race condition handling (if applicable)
- [ ] `test_worktree_path_generation()` - Verify correct paths for agents
- [ ] `test_controller_with_no_config()` - Error handling for missing config

---

### Module: `src/colony/tmux.rs`

**Status**: ❌ No tests

#### Tests Needed
- [ ] `test_check_tmux_available_true()` - Detect installed tmux (mock command)
- [ ] `test_check_tmux_available_false()` - Handle missing tmux
- [ ] `test_create_session_command()` - Generate correct tmux command
- [ ] `test_attach_session_command()` - Verify attach command format
- [ ] `test_kill_session_command()` - Validate kill command
- [ ] `test_send_keys_command()` - Escape special characters correctly
- [ ] `test_session_name_generation()` - Unique session names
- [ ] `test_pane_creation_commands()` - Split panes correctly
- [ ] `test_shell_escaping()` - Prevent injection via agent commands
- [ ] `test_sudo_prompt_for_install()` - Ensure confirmation before sudo

**Mocking Strategy**: Use `mockall` or custom trait wrappers to mock tmux calls

---

### Module: `src/colony/worktree.rs`

**Status**: ❌ No tests

#### Tests Needed
- [ ] `test_create_worktree()` - Create valid git worktree
- [ ] `test_worktree_already_exists()` - Handle existing worktree gracefully
- [ ] `test_cleanup_worktree()` - Remove worktree correctly
- [ ] `test_worktree_with_custom_branch()` - Use specific branch for worktree
- [ ] `test_worktree_path_validation()` - Reject invalid paths
- [ ] `test_git_not_available()` - Error handling when git missing
- [ ] `test_not_a_git_repo()` - Error when run outside git repo
- [ ] `test_worktree_with_uncommitted_changes()` - Handle dirty working tree

**Mocking Strategy**: Use `tempfile::TempDir` for real git repos in tests

---

### Module: `src/colony/tasks/board.rs` & `tasks/queue.rs`

**Status**: ❌ No tests

#### Tests Needed
- [ ] `test_create_task()` - Create task with valid data
- [ ] `test_task_status_transitions()` - Pending → Claimed → InProgress → Completed
- [ ] `test_claim_task()` - Agent claims available task
- [ ] `test_cannot_claim_completed_task()` - Prevent invalid claims
- [ ] `test_task_dependencies()` - Block task until dependency resolves
- [ ] `test_circular_dependency_detection()` - Reject circular deps
- [ ] `test_task_priority_ordering()` - Higher priority tasks first
- [ ] `test_task_persistence()` - Save/load tasks from filesystem
- [ ] `test_task_file_corruption()` - Handle corrupted task files
- [ ] `test_list_tasks_by_status()` - Filter by pending/claimed/completed
- [ ] `test_list_tasks_by_agent()` - Show agent-specific tasks
- [ ] `test_cancel_task()` - Cancel in-progress task
- [ ] `test_delete_task()` - Remove task from board
- [ ] `test_task_blocking()` - Mark task as blocked with reason
- [ ] `test_unblock_task()` - Resume blocked task
- [ ] `test_concurrent_task_claims()` - Prevent race conditions

---

### Module: `src/colony/messaging.rs`

**Status**: ❌ No tests

#### Tests Needed
- [ ] `test_send_message()` - Send message between agents
- [ ] `test_receive_message()` - Retrieve messages for agent
- [ ] `test_message_queue_ordering()` - FIFO message delivery
- [ ] `test_message_persistence()` - Messages saved to disk
- [ ] `test_broadcast_message()` - Message to all agents
- [ ] `test_message_filtering_by_recipient()` - Only retrieve own messages
- [ ] `test_message_serialization()` - JSON format correctness
- [ ] `test_empty_message_queue()` - Handle no messages gracefully
- [ ] `test_message_file_corruption()` - Recover from bad JSON
- [ ] `test_large_message_payload()` - Handle large message content
- [ ] `test_special_characters_in_message()` - Escape quotes, newlines, etc.

---

### Module: `src/colony/init.rs`, `start.rs`, `stop.rs`, `destroy.rs`, etc.

**Status**: ❌ No tests

#### Tests Needed (per command module)
- [ ] `test_init_creates_config()` - Initialize colony.yml template
- [ ] `test_init_already_exists()` - Skip if config exists
- [ ] `test_start_all_agents()` - Start agents defined in config
- [ ] `test_start_specific_agent()` - Start single agent by ID
- [ ] `test_start_with_missing_config()` - Error handling
- [ ] `test_stop_running_agent()` - Gracefully stop agent
- [ ] `test_stop_nonexistent_agent()` - Handle missing agent
- [ ] `test_destroy_cleanup()` - Remove all worktrees and state
- [ ] `test_destroy_confirmation()` - Require user confirmation (mocked)
- [ ] `test_status_display()` - Show agent statuses correctly
- [ ] `test_logs_retrieval()` - Fetch logs for specific agent
- [ ] `test_attach_to_session()` - Generate correct tmux attach command

---

### Module: `src/colony/tui/`

**Status**: ❌ No tests

#### Tests Needed
- [ ] `test_app_initialization()` - Create TUI app state
- [ ] `test_event_handling()` - Process keyboard input
- [ ] `test_ui_rendering()` - Verify ratatui widget layout (snapshot tests)
- [ ] `test_data_refresh()` - Update agent status in real-time
- [ ] `test_exit_tui()` - Handle 'q' key press
- [ ] `test_tui_with_no_agents()` - Display empty state
- [ ] `test_tui_with_many_agents()` - Scrollable list for 50+ agents

**Note**: TUI testing is challenging; consider snapshot testing or visual regression tools

---

## Integration Tests

**Location**: `tests/` (root-level directory)

### Test Suite: `tests/integration_config.rs`

#### Tests Needed
- [ ] `test_load_config_from_file()` - Read colony.yml from disk
- [ ] `test_write_and_reload_config()` - Persist and reload config
- [ ] `test_config_with_includes()` - Support YAML includes (if added later)

---

### Test Suite: `tests/integration_state.rs`

#### Tests Needed
- [ ] `test_full_state_lifecycle()` - Create, update, persist, reload state
- [ ] `test_state_migration()` - Handle old state.json versions (if schema changes)
- [ ] `test_concurrent_state_access()` - File locking (if implemented)
- [ ] `test_state_backup_on_corruption()` - Auto-backup corrupted files

---

### Test Suite: `tests/integration_tasks.rs`

#### Tests Needed
- [ ] `test_task_workflow()` - Create → Claim → Progress → Complete
- [ ] `test_task_dependencies_resolved()` - Task B blocked until Task A completes
- [ ] `test_multiple_agents_claiming_tasks()` - No duplicate claims
- [ ] `test_task_persistence_across_restarts()` - Tasks survive colony restart

---

### Test Suite: `tests/integration_messaging.rs`

#### Tests Needed
- [ ] `test_agent_to_agent_message()` - Send and receive between two agents
- [ ] `test_broadcast_to_all_agents()` - All agents receive broadcast
- [ ] `test_message_persistence()` - Messages survive colony restart
- [ ] `test_message_ordering()` - FIFO delivery guarantee

---

### Test Suite: `tests/integration_worktree.rs`

#### Tests Needed
- [ ] `test_create_multiple_worktrees()` - 5+ agents with isolated worktrees
- [ ] `test_worktree_cleanup_on_destroy()` - All worktrees removed
- [ ] `test_worktree_branch_isolation()` - Each agent on separate branch
- [ ] `test_custom_directory_instead_of_worktree()` - Use custom path for agent

---

### Test Suite: `tests/integration_tmux.rs`

#### Tests Needed
- [ ] `test_create_tmux_session()` - Start real tmux session in CI (if tmux available)
- [ ] `test_attach_and_detach()` - Attach to session and detach
- [ ] `test_kill_tmux_session()` - Clean up session
- [ ] `test_multiple_panes()` - Create 3+ panes for different agents

**Note**: These tests may be skipped in CI if tmux unavailable; use `#[ignore]` or conditional compilation

---

## End-to-End Tests

**Location**: `tests/e2e/`

### Approach

1. Use `assert_cmd` crate to invoke CLI binary
2. Create temporary git repos and colony configs
3. Verify stdout/stderr output and exit codes
4. Check filesystem state (logs, tasks, messages)

### Test Suite: `tests/e2e/test_cli_workflow.rs`

#### Tests Needed
- [ ] `test_full_colony_lifecycle()` - init → start → status → stop → destroy
- [ ] `test_init_command()` - `colony init` creates colony.yml
- [ ] `test_start_command()` - `colony start` launches agents
- [ ] `test_status_command()` - `colony status` shows running agents
- [ ] `test_broadcast_command()` - `colony broadcast "hello"` delivers to all
- [ ] `test_logs_command()` - `colony logs agent-1` shows log file
- [ ] `test_stop_specific_agent()` - `colony stop agent-1` stops one agent
- [ ] `test_stop_all_agents()` - `colony stop` stops all
- [ ] `test_destroy_command()` - `colony destroy` cleans up
- [ ] `test_task_create_and_claim()` - Create task via CLI, claim it
- [ ] `test_task_complete_workflow()` - Create → Claim → Progress → Complete
- [ ] `test_attach_to_tmux()` - `colony attach` (if tmux available)
- [ ] `test_tui_launch()` - `colony tui` starts (exit immediately)

---

### Test Suite: `tests/e2e/test_error_handling.rs`

#### Tests Needed
- [ ] `test_missing_config_error()` - Run commands without colony.yml
- [ ] `test_invalid_yaml_error()` - Malformed colony.yml
- [ ] `test_duplicate_agent_id_error()` - Reject duplicate IDs
- [ ] `test_nonexistent_agent_error()` - Stop/logs for missing agent
- [ ] `test_tmux_not_available_error()` - Handle missing tmux
- [ ] `test_git_not_available_error()` - Handle missing git
- [ ] `test_not_in_git_repo_error()` - Run outside git repo

---

### Test Suite: `tests/e2e/test_multi_agent.rs`

#### Tests Needed
- [ ] `test_start_multiple_agents()` - Start 5 agents simultaneously
- [ ] `test_agents_in_isolation()` - Each agent has separate worktree
- [ ] `test_inter_agent_messaging()` - Agent A sends to Agent B
- [ ] `test_task_queue_with_multiple_agents()` - 3 agents claim 10 tasks
- [ ] `test_agent_failure_isolation()` - One agent failing doesn't affect others

---

## Performance Tests

**Location**: `tests/performance/` or `benches/` (using Criterion)

### Benchmarks Needed

#### Benchmark: `bench_config_parsing.rs`
- [ ] `bench_parse_small_config()` - 5 agents
- [ ] `bench_parse_large_config()` - 100 agents
- [ ] `bench_serialize_config()` - Write config to YAML

#### Benchmark: `bench_task_operations.rs`
- [ ] `bench_create_tasks()` - Create 1000 tasks
- [ ] `bench_claim_task()` - Claim task from queue of 1000
- [ ] `bench_list_tasks()` - List all tasks (1000 items)
- [ ] `bench_task_dependencies()` - Resolve 100 tasks with 10 deps each

#### Benchmark: `bench_messaging.rs`
- [ ] `bench_send_message()` - Send 1000 messages
- [ ] `bench_receive_messages()` - Read 1000 messages for one agent

#### Stress Tests
- [ ] `test_100_agents()` - Start 100 agents (if tmux can handle it)
- [ ] `test_10000_tasks()` - Create and manage 10k tasks
- [ ] `test_concurrent_claims()` - 50 agents claiming tasks simultaneously
- [ ] `test_memory_usage()` - Monitor RSS with 50 agents running
- [ ] `test_disk_usage()` - Check `.colony/` size with 100 agents

---

## Security Tests

**Location**: `tests/security/` or integrated into unit tests

### Test Suite: `tests/security/test_input_validation.rs`

#### Tests Needed
- [ ] `test_path_traversal_in_agent_id()` - Reject `../../../etc/passwd`
- [ ] `test_shell_injection_in_messages()` - Escape `$(rm -rf /)` in messages
- [ ] `test_shell_injection_in_task_content()` - Escape backticks, semicolons
- [ ] `test_yaml_bomb_protection()` - Reject configs with anchor bombs
- [ ] `test_max_config_size()` - Reject configs >10MB
- [ ] `test_agent_id_alphanumeric()` - Only allow `[a-zA-Z0-9_-]`
- [ ] `test_sanitize_log_paths()` - Prevent log file path traversal
- [ ] `test_message_content_escaping()` - JSON/shell escape special chars

---

### Test Suite: `tests/security/test_permissions.rs`

#### Tests Needed
- [ ] `test_state_file_permissions()` - Ensure state.json is not world-readable (if sensitive)
- [ ] `test_log_file_permissions()` - Check log file perms
- [ ] `test_worktree_permissions()` - Verify worktree directory perms

---

### Fuzzing (Optional but Recommended)

Use `cargo-fuzz` to fuzz critical parsers:
- [ ] Fuzz YAML config parser
- [ ] Fuzz JSON state parser
- [ ] Fuzz message content parser
- [ ] Fuzz task content parser

---

## Test Infrastructure

### Helper Modules

Create `tests/common/mod.rs` with utilities:

```rust
// tests/common/mod.rs
use tempfile::TempDir;
use std::path::PathBuf;

pub struct TestEnv {
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
    pub colony_dir: PathBuf,
}

impl TestEnv {
    pub fn new() -> Self {
        // Create temp directory with .git and colony.yml
    }

    pub fn write_config(&self, yaml: &str) {
        // Write colony.yml
    }

    pub fn init_git_repo(&self) {
        // Initialize git repo in temp dir
    }

    pub fn cleanup(&self) {
        // Clean up resources
    }
}

pub fn create_test_agent(id: &str) -> Agent {
    // Helper to create test agents
}

pub fn create_test_config(num_agents: usize) -> Config {
    // Helper to create configs
}
```

### Mocking Strategy

Use these crates for mocking:
- `mockall` - Mock traits and structs
- `mockito` - Mock HTTP servers (if needed later)
- `assert_cmd` - Test CLI commands
- `predicates` - Assert on command output

Example:
```rust
#[cfg(test)]
use mockall::mock;

mock! {
    pub TmuxManager {}
    impl TmuxInterface for TmuxManager {
        fn is_available(&self) -> bool;
        fn create_session(&self, name: &str) -> Result<()>;
    }
}
```

### Test Data Fixtures

Create `tests/fixtures/` with example configs:
- `simple_colony.yml` - 2 agents
- `large_colony.yml` - 50 agents
- `invalid_config.yml` - Malformed YAML
- `duplicate_ids.yml` - Duplicate agent IDs

---

## CI/CD Integration

### Current CI Workflow (`.github/workflows/ci.yml`)

✅ Already includes:
- `cargo test --verbose` on Ubuntu and macOS
- `cargo clippy -- -D warnings`
- `cargo fmt --all -- --check`

### Enhancements Needed

#### 1. Test Coverage Reporting

Add to `.github/workflows/ci.yml`:
```yaml
- name: Install cargo-tarpaulin
  run: cargo install cargo-tarpaulin

- name: Generate coverage report
  run: cargo tarpaulin --out Xml --output-dir coverage

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: coverage/cobertura.xml
```

#### 2. Separate Test Jobs

Split tests into parallel jobs:
- `unit-tests` - Fast unit tests
- `integration-tests` - Slower integration tests
- `e2e-tests` - Full CLI workflows (requires tmux)
- `security-tests` - Security and fuzzing tests

Example:
```yaml
jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test --lib

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test --test '*'

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - name: Install tmux
        run: sudo apt-get install -y tmux
      - run: cargo test --test e2e_*
```

#### 3. Performance Regression Detection

Add benchmark job:
```yaml
  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - run: cargo bench --no-fail-fast
      - name: Store benchmark results
        uses: benchmark-action/github-action-benchmark@v1
```

#### 4. Nightly Fuzz Testing

Create `.github/workflows/fuzz.yml`:
```yaml
name: Fuzz Testing
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      - name: Run fuzz tests
        run: cargo fuzz run --all -- -max_total_time=3600  # 1 hour
```

---

## Testing Tools & Dependencies

### Add to `Cargo.toml` `[dev-dependencies]`

```toml
[dev-dependencies]
tempfile = "3.8"               # Already present
assert_cmd = "2.0"             # CLI testing
predicates = "3.0"             # Output assertions
mockall = "0.12"               # Mocking
mockito = "1.2"                # HTTP mocking (if needed)
proptest = "1.4"               # Property-based testing
criterion = "0.5"              # Benchmarking
cargo-tarpaulin = "0.27"       # Code coverage
insta = "1.34"                 # Snapshot testing (for TUI)
serial_test = "3.0"            # Serialize tests that conflict
```

### Install System Tools

For local development and CI:
```bash
cargo install cargo-tarpaulin  # Coverage
cargo install cargo-fuzz       # Fuzzing
cargo install cargo-audit      # Security audits (already in CI)
```

---

## Test Execution Strategy

### Local Development

Run tests incrementally during development:
```bash
# Fast feedback loop
cargo test --lib  # Unit tests only

# Before commit
cargo test --all  # All tests

# Weekly (slow)
cargo test --release --all-features
cargo bench
```

### Pre-commit Hooks

Create `.git/hooks/pre-commit` (or use `pre-commit` framework):
```bash
#!/bin/bash
cargo test --lib --quiet || exit 1
cargo clippy -- -D warnings || exit 1
cargo fmt --all -- --check || exit 1
```

### CI Workflow

1. **PR Checks** (required to pass):
   - Unit tests
   - Integration tests
   - Linting (clippy)
   - Formatting (rustfmt)
   - Security audit

2. **Nightly Jobs** (informational):
   - E2E tests (may be flaky due to tmux)
   - Performance benchmarks
   - Fuzz testing
   - Coverage reporting

---

## Coverage Goals

### Initial Targets (3 months)

| Category | Current | Target | Notes |
|----------|---------|--------|-------|
| **Unit Tests** | ~5% (config only) | 70% | Core logic coverage |
| **Integration Tests** | 0% | 50% | Module interactions |
| **E2E Tests** | 0% | 30% | Critical paths |
| **Overall Coverage** | ~5% | 65% | Measured by tarpaulin |

### Long-term Targets (6 months)

| Category | Target | Notes |
|----------|--------|-------|
| **Unit Tests** | 85% | Exclude trivial getters/setters |
| **Integration Tests** | 70% | All major workflows |
| **E2E Tests** | 50% | All CLI commands |
| **Overall Coverage** | 80% | Industry standard for critical tools |

### Excluded from Coverage

- Generated code (if any)
- Main function boilerplate
- Display/Debug trait implementations (unless complex)
- Trivial getters/setters

---

## Known Gaps & Future Work

### Current Limitations

1. **No Async Test Utilities**: Need helpers for testing async code with tokio
2. **TUI Testing**: Visual testing is minimal; consider adding screenshot comparisons
3. **External Dependencies**: tmux and git are not mocked in integration tests
4. **Concurrency Testing**: Limited testing of race conditions
5. **Chaos Testing**: No failure injection (e.g., simulating disk full, network errors)

### Future Enhancements

#### Phase 2: Advanced Testing (6-12 months)
- [ ] Property-based testing with `proptest` for data structures
- [ ] Mutation testing with `cargo-mutants` to verify test quality
- [ ] Snapshot testing for TUI with `insta`
- [ ] Docker-based E2E tests with isolated tmux environments
- [ ] Performance regression alerts in CI
- [ ] Test flakiness detection and reporting

#### Phase 3: Production Hardening (12+ months)
- [ ] Chaos engineering tests (kill agents randomly, corrupt files)
- [ ] Load testing with 1000+ agents
- [ ] Compatibility testing across tmux versions
- [ ] Backward compatibility tests for old colony.yml formats
- [ ] Security penetration testing (external audit)

---

## Test Maintenance Guidelines

### Writing Good Tests

1. **Naming Convention**: `test_<module>_<scenario>_<expected_outcome>`
   - Good: `test_config_duplicate_ids_returns_error`
   - Bad: `test_config_1`

2. **AAA Pattern**: Arrange, Act, Assert
   ```rust
   #[test]
   fn test_agent_status_transition() {
       // Arrange
       let mut agent = Agent::new("test-agent");

       // Act
       agent.set_status(AgentStatus::Running);

       // Assert
       assert_eq!(agent.status, AgentStatus::Running);
   }
   ```

3. **Test One Thing**: Each test should verify one behavior
4. **Avoid Test Interdependence**: Tests must not rely on execution order
5. **Use Descriptive Assertions**: Prefer `assert_eq!(actual, expected, "Agent status should transition to Running")` over `assert!(condition)`

### Refactoring Tests

- Extract common setup to helper functions
- Use fixtures for test data
- Avoid copy-paste; DRY principle applies to tests too
- Keep tests close to the code they test (inline modules for unit tests)

### Handling Flaky Tests

1. Identify root cause (timing, external deps, race conditions)
2. Add `#[ignore]` with a comment explaining why
3. Fix determinism issues (mock time, mock external calls)
4. Use `#[serial]` from `serial_test` crate if tests must run sequentially

---

## Success Metrics

### Definition of Done for Testing

- ✅ All new code includes unit tests (enforced in code review)
- ✅ Critical paths have integration tests
- ✅ All CLI commands have at least one E2E test
- ✅ Coverage >65% overall, >70% for core modules
- ✅ CI passes all tests on PRs
- ✅ No known flaky tests in main branch
- ✅ Security tests pass (input validation, injection prevention)
- ✅ Benchmarks show no performance regressions

### Review Checklist for PRs

- [ ] New code has corresponding tests
- [ ] Tests follow naming conventions
- [ ] Tests are independent and deterministic
- [ ] Coverage has not decreased
- [ ] All CI checks pass
- [ ] Documentation updated if behavior changed

---

## Appendix: Test Examples

### Example 1: Unit Test with Mock

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub FileSystem {}
        impl FileSystem for FileSystem {
            fn read(&self, path: &str) -> Result<String>;
            fn write(&self, path: &str, content: &str) -> Result<()>;
        }
    }

    #[test]
    fn test_load_config_success() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_read()
            .with(eq("colony.yml"))
            .returning(|_| Ok("agents:\n  - id: test\n    role: developer".to_string()));

        let config = Config::load_with_fs(&mock_fs).unwrap();
        assert_eq!(config.agents.len(), 1);
        assert_eq!(config.agents[0].id, "test");
    }
}
```

### Example 2: Integration Test

```rust
// tests/integration_state.rs
use cc_colony::colony::{Controller, Config};
use tempfile::TempDir;

#[test]
fn test_state_persistence() {
    let temp = TempDir::new().unwrap();
    let config = Config::default();

    // Create controller and add agent
    let mut controller = Controller::new(config, temp.path());
    controller.add_agent("agent-1", "developer").unwrap();
    controller.save_state().unwrap();

    // Reload controller from disk
    let loaded_controller = Controller::load(temp.path()).unwrap();
    assert_eq!(loaded_controller.agents().len(), 1);
    assert_eq!(loaded_controller.agents()[0].id, "agent-1");
}
```

### Example 3: E2E Test

```rust
// tests/e2e/test_cli_workflow.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_init_command_creates_config() {
    let temp = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("colony").unwrap();
    cmd.current_dir(&temp)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized colony"));

    // Verify colony.yml was created
    assert!(temp.path().join("colony.yml").exists());
}
```

---

## Conclusion

This testing plan provides a roadmap to achieve comprehensive test coverage for cc-colony. By incrementally implementing unit, integration, E2E, performance, and security tests, we will build confidence in the tool's reliability and maintainability.

**Next Steps**:
1. Start with unit tests for `config.rs`, `agent.rs`, and `controller.rs`
2. Add integration tests for state persistence and task workflows
3. Implement basic E2E tests for CLI commands
4. Set up coverage reporting in CI
5. Iterate based on coverage gaps and bug reports

**Ownership**: Testing is a team responsibility. All contributors should write tests for their code, and reviewers should enforce test requirements in PRs.

---

**Document Version**: 1.0
**Last Updated**: 2025-11-11
**Maintainer**: CC-Colony Development Team
