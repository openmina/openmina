# Git Workflow and PR Policy

This document outlines the git workflow and pull request policy used in the
OpenMina repository.

## Branch Management

### Main Branches

- **`develop`** - Main integration branch for active development
- **`main`** - Stable branch that receives periodic merges from develop

**Note**: Unlike the OCaml Mina node which has a `compatible` branch, OpenMina
does not maintain a compatibility branch because we haven't had to support two
different protocol versions simultaneously.

### Branch Naming Conventions

**Feature Branches:**

- `feat/feature-name` - New features and enhancements
- `fix/issue-description` - Bug fixes and corrections
- `tweaks/component-name` - Improvements and optimizations
- `chore/description` - Maintenance, CI, and tooling updates

**Release Branches:**

- `prepare-release/vX.Y.Z` - Release preparation branches

**Examples:**

```
feat/error-sink-service
fix/initial-peers-resolve-failure
tweaks/yamux
chore/ci-runner-images
prepare-release/v0.16.0
```

## Pull Request Development Workflow

### 1. Development Phase

- **Create feature branch** from latest `develop`
- **Make incremental commits** without squashing
- **Use descriptive commit messages** following conventional commits format
- **Push all commits** to remote branch regularly
- **Rebase branch** regularly to stay current with develop
- **Use WIP/tmp commits** for work-in-progress saves

### 2. Review Phase

- **Keep linear intermediary commits** during review process
- **All commits remain visible** to ease reviewers' job
- **Reviewers can see progression** of changes and iterations
- **Address review feedback** with additional commits
- **Regular rebasing** to keep branch current with develop

### 3. Pre-Merge Phase

- **Code review** - PRs should ideally be reviewed by another team member
- **Squash related commits** that don't make sense alone:
  - Fixup commits (`fixup tests`, `more refactor`)
  - WIP commits (`tmp`, `WIP`)
  - Incremental improvements to the same feature
  - Review feedback commits
- **Preserve meaningful commits** that represent separate logical changes
- **Final rebase** against latest develop
- **Clean history for posterity** - helps when checking history after merge

### 4. Merge Phase

- **Merge with merge commit** (no fast-forward)
- **Delete feature branch** using GitHub's UI after merge

## Commit Message Format

Follow conventional commits format: `type(scope): description`

**Common Types:**

- `feat` - New features
- `fix` - Bug fixes
- `chore` - Maintenance tasks
- `refactor` - Code restructuring
- `tweak` - Minor improvements

**Examples:**

```
feat(yamux): Split incoming frame handling into multiple actions
fix(p2p): Do not fail when an initial peer address cannot be resolved
chore(ci): Upgrade jobs to use Ubuntu 22.04
tweak(yamux): Set max window size to 256kb
```

## Commit Squashing Policy

### Purpose

- **During review**: Keep all commits visible to help reviewers understand the
  development process
- **Before merge**: Squash commits to create clean history for future reference
  and debugging

### When to Squash

- **Fixup commits** - Commits that fix issues in previous commits
- **WIP commits** - Temporary work-in-progress commits
- **Incremental improvements** - Multiple commits that refine the same feature
- **Review feedback** - Commits that address review comments

### When NOT to Squash

- **Separate logical changes** - Commits that represent distinct features or
  fixes
- **Different components** - Changes that affect different parts of the system
- **Meaningful progression** - Commits that show logical development steps

### Examples

**Single-commit PRs** (no squashing needed):

```
fix(p2p): Do not fail when an initial peer address cannot be resolved
```

**Multi-commit PRs** (preserve logical units):

```
chore(ci): Upgrade jobs to use Ubuntu 22.04
chore(ci): Install libssl3 from bookworm in mina bullseye images
```

**Complex PRs** (squash related work):

```
Before squashing:
- feat(yamux): Set max window size to 256kb
- tweak(yamux): Simplify reducer a bit
- tweak(yamux): Simplify reducer a bit more
- feat(yamux): Split incoming frame handling into multiple actions
- feat(yamux): Add tests
- fixup tests
- more refactor
- fixup tests (clippy)

After squashing:
- feat(yamux): Improve reducer and add comprehensive tests
- feat(yamux): Split incoming frame handling into multiple actions
```

## Best Practices

1. **Rebase regularly** - Keep feature branches up-to-date with develop
2. **Commit often** - Make small, focused commits during development
3. **Clean before merge** - Ensure final commit history is logical and readable
4. **Descriptive messages** - Write clear, specific commit messages
5. **Review history** - Check that squashed commits tell a coherent story
6. **Test before merge** - Ensure all commits in the final history build and
   pass tests

## Merge Strategy

- **Merge commits** are created for all PRs:
  `Merge pull request #XXXX from openmina/branch-name`
- **No fast-forward merges** - Merge commits preserve PR context and history
- **Rebase before merge** - Branches are rebased to develop before merging
- **Delete merged branches** - Use GitHub's UI to delete feature branches after
  successful merge

This workflow balances development flexibility with clean version history,
allowing for iterative development while ensuring the final merged result has a
clear, logical commit structure.
