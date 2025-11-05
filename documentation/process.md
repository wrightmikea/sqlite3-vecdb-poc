# Development Process

This document defines the development workflow, quality standards, and best practices for VectDB.

## Core Principles

1. **Test-Driven Development (TDD)**: Write tests before implementation
2. **Quality First**: No compromises on code quality or test coverage
3. **Documentation**: Code and commits are well-documented
4. **Incremental Progress**: Small, focused commits with clear purposes
5. **Continuous Integration**: All code must pass quality checks

## Development Workflow

### Before Starting Work

1. **Check Documentation**: Review relevant documentation files:
   - `documentation/plan.md` - Current priorities and next steps
   - `documentation/architecture.md` - System architecture
   - `documentation/design.md` - Component designs
   - `CLAUDE.md` - Development guidelines

2. **Understand Requirements**: Clarify the feature/fix requirements
   - What problem are we solving?
   - What are the acceptance criteria?
   - Are there related issues or PRs?

3. **Plan Approach**: Think through the solution
   - What components need to change?
   - What tests are needed?
   - Are there breaking changes?

### Test-Driven Development (TDD)

VectDB follows the **Red-Green-Refactor** cycle:

#### 1. Red Phase: Write Failing Tests

```bash
# Create test that defines expected behavior
# Test should fail initially (red)

# Example for new feature
#[test]
fn test_new_feature() {
    // Arrange: Set up test conditions
    let input = setup_test_data();

    // Act: Call the feature
    let result = new_feature(input);

    // Assert: Verify expected outcome
    assert_eq!(result, expected_value);
}

# Run tests to confirm they fail
cargo test test_new_feature
# Should fail with clear error message
```

#### 2. Green Phase: Make Tests Pass

```bash
# Implement minimal code to make test pass
# Focus on making it work, not perfect

# Implement feature
impl MyStruct {
    pub fn new_feature(&self, input: Input) -> Output {
        // Minimal implementation
    }
}

# Run tests to confirm they pass
cargo test test_new_feature
# Should pass (green)
```

#### 3. Refactor Phase: Improve Code Quality

```bash
# Now that tests pass, improve the implementation
# - Remove duplication
# - Improve naming
# - Simplify logic
# - Add documentation

# Run full test suite after each refactoring
cargo test

# Ensure tests still pass after refactoring
```

### Testing Requirements

#### Test Coverage Standards

All new code must have:
- **Unit tests**: Test individual functions/methods in isolation
- **Integration tests**: Test component interactions
- **Edge cases**: Test boundary conditions and error handling
- **Documentation tests**: Verify code examples in doc comments

#### Test Categories

```rust
// 1. Unit tests (same file as implementation)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Test happy path
    }

    #[test]
    fn test_error_handling() {
        // Test error conditions
    }

    #[test]
    fn test_edge_cases() {
        // Test boundaries
    }
}

// 2. Integration tests (tests/ directory)
// tests/integration_test.rs
#[tokio::test]
async fn test_end_to_end_workflow() {
    // Test full workflow
}

// 3. Doc tests (in documentation comments)
/// # Examples
/// ```
/// use vectdb::Document;
/// let doc = Document::new("test.txt".to_string(), "content");
/// assert_eq!(doc.source, "test.txt");
/// ```
```

#### Testing Tools and Helpers

```rust
// Use in-memory database for tests
let store = VectorStore::in_memory()?;

// Use tempfile for file-based tests
use tempfile::NamedTempFile;
let temp = NamedTempFile::new()?;

// Mock Ollama when needed (or handle gracefully)
if !ollama.health_check().await? {
    // Skip or use mock data
    return Ok(());
}
```

### UI Testing with MCP/Playwright

For web interface changes, use MCP/Playwright to verify UI functionality:

```bash
# 1. Start the server
./scripts/serve.sh &

# 2. Use MCP/Playwright to test
# - Navigate to pages
# - Fill forms
# - Click buttons
# - Verify results
# - Take screenshots

# 3. Stop the server
kill $SERVER_PID
```

Example workflow:
```rust
// In test or manual verification
mcp__playwright__playwright_navigate("http://localhost:3000")
mcp__playwright__playwright_fill("#query", "test search")
mcp__playwright__playwright_click("button")
mcp__playwright__playwright_screenshot("test-search-results")
```

## Quality Checks

### Pre-Commit Quality Process

**IMPORTANT**: Before **every** commit, run the following checks:

#### 1. Format Code

```bash
# Run rustfmt to ensure consistent formatting
cargo fmt

# Verify no changes needed
cargo fmt -- --check
```

#### 2. Fix Clippy Warnings

```bash
# Run clippy with strict settings
cargo clippy --all-targets --all-features -- -D warnings

# Fix all warnings (not just errors)
# Clippy helps catch common mistakes and improve code quality
```

Common clippy fixes:
- Remove unused imports
- Use `is_empty()` instead of `.len() == 0`
- Use `copied()` instead of `.map(|x| *x)`
- Remove needless borrows

#### 3. Run Tests

```bash
# Run full test suite
cargo test

# Run tests with output (if debugging)
cargo test -- --nocapture

# Run specific test
cargo test test_name

# All tests must pass
```

#### 4. Build

```bash
# Verify release build works
cargo build --release

# Or use build script
./scripts/build.sh

# Check for warnings during compilation
# Fix any warnings (not just errors)
```

#### 5. Verify .gitignore

Before committing:
- Check `git status` for unexpected files
- Ensure build artifacts are excluded (target/, *.log, *.db)
- Add new patterns to .gitignore if needed
- Verify example/demo files are properly excluded

```bash
# Review what will be committed
git status

# Check for large or binary files
git diff --stat

# Ensure no secrets, credentials, or personal data
```

#### 6. Update Documentation

If your changes affect:
- **README.md**: Update usage examples, commands, features
- **CLAUDE.md**: Update development guidelines
- **documentation/*.md**: Update relevant design/architecture docs
- **Code comments**: Add/update doc comments for public APIs
- **CHANGELOG** (if we add one): Document breaking changes

### Quality Checklist

Before committing, verify:

- [ ] Code formatted with `cargo fmt`
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] All tests pass (`cargo test`)
- [ ] Release build succeeds (`cargo build --release`)
- [ ] .gitignore is correct
- [ ] Documentation updated
- [ ] Commit message is clear and descriptive
- [ ] No debug/temporary code left in
- [ ] No commented-out code (unless with explanation)
- [ ] No `println!` or `dbg!` (use `tracing` instead)

## Commit Guidelines

### Commit Message Format

```
<type>: <summary in 50 chars or less>

<body - explain WHAT and WHY, not HOW>
<body - wrap at 72 characters>

<footer - reference issues, breaking changes>

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

### Commit Types

- **feat**: New feature
- **fix**: Bug fix
- **refactor**: Code restructuring (no behavior change)
- **test**: Adding or updating tests
- **docs**: Documentation changes
- **style**: Formatting, naming (no logic change)
- **perf**: Performance improvement
- **chore**: Build process, dependencies, tooling

### Good Commit Messages

**Good**:
```
feat: Add PDF ingestion support

Implement PDF document parsing using pdf-extract crate.
Handles text extraction, metadata, and page boundaries.

Resolves #42
```

**Bad**:
```
fix stuff
```

### Commit Frequency

- Commit frequently (after each Red-Green-Refactor cycle)
- Each commit should be a logical unit of work
- Keep commits focused (one feature/fix per commit)
- Don't commit broken code (all tests must pass)

## Code Review Process

### Self-Review Checklist

Before pushing:

1. **Review your own changes**:
   ```bash
   git diff main...HEAD
   ```

2. **Check for**:
   - Unnecessary changes (formatting in unrelated files)
   - Debug code left in
   - TODO/FIXME comments (address or document)
   - Hardcoded values (should be configurable)
   - Error messages are helpful
   - Comments explain WHY, not WHAT

3. **Test edge cases**:
   - Empty inputs
   - Large inputs
   - Invalid inputs
   - Concurrent access (if applicable)

### Collaboration Guidelines

When working with others:
- Ask clarifying questions
- Suggest, don't demand
- Praise good work
- Be specific in feedback
- Focus on the code, not the person

## Continuous Integration

### Automated Checks (when CI is set up)

All PRs must pass:
- [ ] `cargo fmt -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test`
- [ ] `cargo build --release`
- [ ] Documentation builds successfully

### Local Pre-Push Checks

```bash
# Run all checks locally before pushing
./scripts/check.sh  # (to be created)

# Or manually:
cargo fmt -- --check && \
cargo clippy --all-targets -- -D warnings && \
cargo test && \
cargo build --release
```

## Debugging Process

### When Tests Fail

1. **Read the error message carefully**
   - What assertion failed?
   - What were the actual vs expected values?

2. **Run test with output**:
   ```bash
   cargo test failing_test -- --nocapture
   ```

3. **Add debug logging**:
   ```rust
   use tracing::debug;
   debug!("Value: {:?}", value);
   ```

4. **Use debugger** (if needed):
   ```bash
   rust-lldb target/debug/deps/vectdb-<hash>
   ```

5. **Simplify the test**:
   - Reduce input size
   - Test one thing at a time
   - Isolate the failing component

### When Clippy Fails

1. **Read the explanation**:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

2. **Understand the suggestion**:
   - Why is clippy suggesting this?
   - Is it a real issue?

3. **Fix or allow**:
   ```rust
   // If legitimate reason to ignore:
   #[allow(clippy::lint_name)]
   fn my_function() { }
   ```

4. **Learn from it**: Clippy teaches Rust idioms

## Performance Optimization Process

When optimizing:

1. **Measure first** (don't guess):
   ```bash
   cargo build --release
   time ./target/release/vectdb command
   ```

2. **Profile** (if needed):
   ```bash
   cargo install cargo-flamegraph
   cargo flamegraph --bin vectdb -- command
   ```

3. **Optimize hot paths only**
   - Don't optimize until it's proven necessary
   - Readability > micro-optimizations

4. **Benchmark**:
   ```bash
   cargo bench
   ```

5. **Verify performance gain**:
   - Before/after measurements
   - Document in commit message

## Documentation Process

### Code Documentation

```rust
/// Brief one-line summary
///
/// Longer description explaining the purpose and behavior.
///
/// # Examples
///
/// ```
/// use vectdb::MyStruct;
/// let instance = MyStruct::new();
/// ```
///
/// # Errors
///
/// Returns error if...
///
/// # Panics
///
/// Panics if...
pub fn my_function() { }
```

### README Updates

Update README.md when:
- Adding new commands
- Changing configuration options
- Adding dependencies
- Changing installation process
- Adding features

### Architecture Documentation

Update documentation/*.md when:
- Changing core architecture
- Adding new modules/components
- Changing data models
- Modifying algorithms
- Adding new dependencies

## Release Process

### Pre-Release Checklist

- [ ] All tests pass
- [ ] Documentation is up to date
- [ ] CHANGELOG is updated
- [ ] Version bumped in Cargo.toml
- [ ] Release notes drafted
- [ ] Binaries built for all platforms
- [ ] Demo is updated

### Release Steps

```bash
# 1. Update version
# Edit Cargo.toml

# 2. Update documentation
# Edit documentation/status.md, README.md

# 3. Commit version bump
git commit -m "chore: Release v0.2.0"

# 4. Tag release
git tag -a v0.2.0 -m "Release v0.2.0"

# 5. Build release binaries
./scripts/build.sh

# 6. Push
git push && git push --tags
```

## Emergency Fixes

### Hotfix Process

For critical bugs in production:

1. **Create hotfix branch** from main
2. **Fix the bug** (minimal changes)
3. **Add regression test**
4. **Fast-track review**
5. **Merge and release immediately**

### Rollback Process

If a release has critical issues:

```bash
# Revert to previous version
git revert <bad-commit>

# Or create fix-forward commit
git commit -m "fix: Revert problematic change"
```

## Resources

### Learning Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [TDD in Rust](https://doc.rust-lang.org/book/ch11-00-testing.html)

### Tools

- `cargo fmt`: Code formatting
- `cargo clippy`: Linting
- `cargo test`: Testing
- `cargo bench`: Benchmarking
- `cargo doc`: Documentation generation

### Getting Help

- Check documentation first
- Search issues on GitHub
- Ask in Rust community forums
- Create detailed issue with reproduction steps

## Summary

**Remember**:
1. ✅ Always use TDD (Red-Green-Refactor)
2. ✅ Run quality checks before every commit
3. ✅ Write clear commit messages
4. ✅ Keep documentation up to date
5. ✅ Test edge cases and error handling
6. ✅ Use MCP/Playwright for UI testing
7. ✅ Review your own code first
8. ✅ Learn from clippy and compiler errors

**Quality is not negotiable. Every commit should be production-ready.**
