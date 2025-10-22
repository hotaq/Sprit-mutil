# Test Suite for Sprite CLI

This directory contains the comprehensive test suite for the Sprite multi-agent CLI application.

## Directory Structure

```
tests/
├── unit/                          # Unit tests
│   ├── cli_parsing_test.rs       # CLI argument parsing tests
│   ├── config_test.rs            # Configuration management tests
│   └── mod.rs                    # Unit test module declaration
├── integration/                   # Integration tests
│   ├── init_workflow_test.rs     # Init command integration tests
│   ├── slash_command_test.rs     # Slash command integration tests
│   ├── multi_agent_test.rs       # Multi-agent workflow tests
│   ├── tmux_integration_test.rs  # Tmux integration tests
│   └── mod.rs                    # Integration test module declaration
├── e2e/                          # End-to-end tests
│   ├── complete_workflow_test.rs # Complete workflow tests
│   └── mod.rs                    # E2E test module declaration
├── performance/                   # Performance tests
│   ├── performance_requirements_test.rs # Performance requirement tests
│   └── mod.rs                    # Performance test module declaration
├── mocks/                        # Mock implementations
│   ├── ai_framework_mocks.rs    # AI framework API mocks
│   ├── http_mocks.rs            # HTTP service mocks
│   └── file_system_mocks.rs     # File system mocks
├── common/                       # Common test utilities
│   ├── fixtures.rs              # Test fixtures
│   ├── utils.rs                 # Test utilities
│   └── mod.rs                   # Common module declaration
└── README.md                     # This file
```

## Test Categories

### Unit Tests
Unit tests focus on testing individual functions and modules in isolation:

- **CLI Parsing Tests**: Verify command-line argument parsing works correctly
- **Configuration Tests**: Test configuration loading, saving, and validation
- **Tmux Utilities Tests**: Test tmux-related utility functions

### Integration Tests
Integration tests verify that multiple components work together correctly:

- **Init Workflow Tests**: Test the complete initialization workflow
- **Slash Command Tests**: Test slash command integration with external AI frameworks
- **Multi-Agent Tests**: Test multi-agent coordination and communication
- **Tmux Integration Tests**: Test tmux session management and command execution

### End-to-End Tests
E2E tests verify complete user workflows from start to finish:

- **Complete Workflow Tests**: Test entire user scenarios including initialization, agent creation, command execution, and cleanup

### Performance Tests
Performance tests ensure the application meets performance requirements:

- **Simple Commands**: Must complete in <2 seconds
- **Complex Commands**: Must complete in <5 seconds
- **Large Scale Tests**: Test with larger agent counts

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories
```bash
# Run only unit tests
cargo test --test unit

# Run only integration tests
cargo test --test integration

# Run only performance tests
cargo test --test performance

# Run only E2E tests
cargo test --test e2e
```

### Run with Coverage
```bash
# Install cargo-tarpaulin first
cargo install cargo-tarpaulin

# Run tests with coverage
cargo tarpaulin --out Html --output-dir target/coverage
```

### Run Benchmarks
```bash
# Run performance benchmarks
cargo bench

# Run benchmarks without executing (compilation check)
cargo bench --no-run
```

## Performance Requirements

The following performance requirements are enforced:

- **Simple Commands** (agents list, status, config show): < 2 seconds
- **Complex Commands** (init with many agents, provisioning): < 5 seconds
- **Slash Commands** (speckit operations): < 5 seconds
- **Concurrent Operations**: Multiple simple commands < 3 seconds total

## Test Configuration

### Coverage Requirements
- **Line Coverage**: Minimum 80%
- **Function Coverage**: Minimum 80%
- **Branch Coverage**: Minimum 75%

See `.tarpaulin.toml` for detailed coverage configuration.

### CI/CD Integration
The test suite is integrated with GitHub Actions (`.github/workflows/test.yml`) and includes:

- Multi-Rust version testing (stable, beta)
- Code formatting checks (cargo fmt)
- Linting (cargo clippy)
- Unit and integration tests
- Performance tests
- Security audits

## Mock Implementations

### AI Framework Mocks
The `tests/mocks/ai_framework_mocks.rs` file provides comprehensive mocks for:

- **Claude API**: Mock responses for Claude Sonnet and other Claude models
- **Codex API**: Mock responses for GitHub Copilot/Codex
- **Droid API**: Mock responses for other AI frameworks

These mocks allow testing without actual API calls and provide deterministic behavior.

### HTTP Service Mocks
HTTP service mocks using `wiremock` allow testing external API integrations:

- REST API endpoint mocking
- Response status and body configuration
- Error scenario simulation

### File System Mocks
File system mocks using `tempfile` and `assert_fs` provide:

- Temporary directory creation for isolated tests
- File system assertion utilities
- Cleanup after test completion

## Best Practices

### Test Naming
- Unit tests: `test_<functionality>_<scenario>`
- Integration tests: `test_<module>_<workflow>_workflow`
- E2E tests: `test_complete_<workflow>_workflow`
- Performance tests: `test_<complexity>_commands_performance`

### Test Organization
- Group related tests in modules
- Use descriptive test names
- Include setup and teardown in test fixtures
- Use mocks for external dependencies

### Error Testing
- Test both success and failure scenarios
- Verify error messages are appropriate
- Test edge cases and boundary conditions

### Performance Testing
- Measure actual execution time
- Compare against requirements
- Test with different data sizes
- Include concurrent operation tests

## Adding New Tests

When adding new functionality:

1. **Unit Tests**: Add tests for individual functions in `tests/unit/`
2. **Integration Tests**: Add workflow tests in `tests/integration/`
3. **Performance Tests**: Add performance requirements in `tests/performance/`
4. **Update Documentation**: Update this README and relevant test documentation
5. **Update CI**: Ensure new tests are covered in CI/CD pipeline

## Troubleshooting

### Test Failures
- Check test output for specific error messages
- Verify system dependencies (tmux, git) are installed
- Ensure proper permissions for temporary directories
- Check network connectivity for integration tests

### Performance Test Failures
- Verify system load is reasonable
- Check available disk space for large tests
- Ensure proper system configuration
- Consider test environment differences

### Coverage Issues
- Check `.tarpaulin.toml` configuration
- Verify test files are properly discovered
- Check exclude patterns are not too broad
- Ensure tests actually exercise the code paths