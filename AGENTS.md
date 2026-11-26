# Agent Guidelines for hdf5-rt

Read `README.md` before starting work.

## Development Stage

**Early development** - no backward compatibility required. Remove deprecated code immediately.

## General Guidelines

- Use same language as past conversations (Japanese if previous was Japanese)
- Source code and docs in English
- Workspace crates: `hdf5/`, `hdf5-types/`
- **Bug fixing**: When a bug is discovered, always check related files for similar bugs and propose to the user to inspect them

## Context-Efficient Exploration

- Use Task tool with `subagent_type=Explore` for open-ended exploration
- Use Grep for structure: `pub fn`, `impl.*for`, `^pub (struct|enum|type)`
- Read specific lines with `offset`/`limit` parameters

## Code Style

`cargo fmt` for formatting, `cargo clippy` for linting. Avoid `unwrap()`/`expect()` in library code.

**Always run `cargo fmt --all` before committing changes.**

## Error Handling

- Use `hdf5::Result` and `hdf5::Error` types
- Provide meaningful error context

## Testing

```bash
cargo test                    # Full suite
cargo test --test test_name   # Specific test
cargo test --workspace        # All crates
```

- Private functions: use module-local `src/<module>/tests/*.rs` files per the
  shared [Unit Test Organization rule](https://github.com/tensor4all/tensor4all-agent-rules/blob/main/rules/rust/index.md#unit-test-organization);
  no inline `#[cfg(test)]` module in the source file.
- Integration tests: `tests/` directory
- **Test tolerance changes**: When relaxing test tolerances, always seek explicit user approval before making changes.

## Features

Available features:
- `complex`: Complex number type support (Complex32, Complex64)
- `f16`: Float16 type support
- `runtime-loading`: Runtime library loading via dlopen

## Git Workflow

**Never push/create PR without user approval.**

### Pre-PR Checks

Before creating a PR, always run lint checks locally:

```bash
cargo fmt --all          # Format all code
cargo clippy --workspace # Check for common issues
cargo test --workspace   # Run all tests
```

| Change Type | Workflow |
|-------------|----------|
| Minor fixes | Branch + PR with auto-merge |
| Large features | Worktree + PR with auto-merge |

```bash
# Minor: branch workflow
git checkout -b fix-name && git add -A && git commit -m "msg"
cargo fmt --all && cargo clippy --workspace  # Lint before push
git push -u origin fix-name
gh pr create --base main --title "Title" --body "Desc"
gh pr merge --auto --squash --delete-branch

# Large: worktree workflow
git worktree add ../hdf5-rt-feature -b feature

# Check PR before update
gh pr view <NUM> --json state  # Never push to merged PR

# Monitor CI
gh pr checks <NUM>
gh run view <RUN_ID> --log-failed
```

**Before creating PR**: Verify README.md is accurate (project structure, examples).
