# OpenMina Release Process

This document outlines the release process for OpenMina, including version
management, tagging, and automated Docker image builds.

## Overview

The OpenMina release process involves:

1. Creating a release preparation branch from `develop`
2. Updating version numbers across all Cargo.toml files
3. Updating the changelog with release notes and comparison links
4. Updating Docker Compose files with new version tags
5. Creating a PR to merge release changes to `develop`
6. Creating a PR to merge `develop` to `main`
7. Creating a git tag from `main` with the proper metadata
8. Automated CI/CD workflows that build and publish Docker images

## Branch Strategy

- **develop**: All changes between releases go to the `develop` branch
- **main**: Stable release branch, updated only during releases
- **prepare-release/vX.X.X**: Temporary branch for preparing release changes
- Public releases are always tagged from the `main` branch after merging from
  `develop`
- Internal/non-public patch releases can be tagged directly from `develop`

## Release Cadence

During active development, OpenMina follows a monthly release schedule. At the
end of each month, all changes that have been merged to `develop` are packaged
into a new release. This regular cadence ensures:

- Predictable release cycles for users
- Regular integration of new features and fixes
- Manageable changelog sizes
- Consistent testing and deployment rhythm

## Prerequisites

- All desired changes merged to `develop` branch
- All tests passing on `develop`
- Access to create and push git tags
- Permission to merge to `main` branch

## Release Steps

### 1. Create Release Preparation Branch

Create a new branch from `develop` for preparing the release:

```bash
git checkout develop
git pull origin develop
git checkout -b prepare-release/vX.Y.Z
```

### 2. Update Version Numbers

Use the `versions.sh` script to update all Cargo.toml files with the new
version:

```bash
./versions.sh X.Y.Z
```

This script will:

- Find all Cargo.toml files in the project
- Update the version field in each file (except `mina-p2p-messages/Cargo.toml`
  which is handled manually)
- Display the version changes for each file

### 3. Update Changelog

Update the CHANGELOG.md file following the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format:

1. **Move unreleased changes to new version section**:
   - Change `## [Unreleased]` to `## [X.Y.Z] - YYYY-MM-DD` (use current date)
   - Add a new empty `## [Unreleased]` section at the top

2. **Organize changes by category**:
   - `### Added` - for new features
   - `### Changed` - for changes in existing functionality
   - `### Deprecated` - for soon-to-be removed features
   - `### Removed` - for now removed features
   - `### Fixed` - for bug fixes
   - `### Security` - for vulnerability fixes

3. **Update comparison links at the bottom**:
   - Add a new comparison link for the release:
     ```markdown
     [X.Y.Z]: https://github.com/openmina/openmina/compare/vA.B.C...vX.Y.Z
     ```
   - Update the `[Unreleased]` link to compare from the new version to develop:
     ```markdown
     [Unreleased]: https://github.com/openmina/openmina/compare/vX.Y.Z...develop
     ```

   The release link compares the previous version tag with the new version tag.

### 4. Update Docker Compose Files

Update the image versions in all docker-compose files. For example, in
`docker-compose.local.producers.yml`:

```yaml
image: openmina/openmina:X.Y.Z
```

### 5. Commit Version Changes

Commit all the version updates, changelog, and docker-compose changes:

```bash
git add CHANGELOG.md
git add Cargo.toml */Cargo.toml */*/Cargo.toml  # Add all Cargo.toml files
git add Cargo.lock
git add docker-compose*.yml
git commit -m "chore: Prepare release vX.Y.Z"
```

**Note**: Avoid using `git add .` to prevent accidentally committing unrelated
files.

### 6. Create PR to Develop

Push the release preparation branch and create a PR to merge it into `develop`:

```bash
git push origin prepare-release/vX.Y.Z
```

Then create a PR from `prepare-release/vX.Y.Z` to `develop` on GitHub. Once
approved and merged, continue with the next steps.

### 7. Create PR to Main

After the release preparation has been merged to `develop`, create a PR to merge
`develop` into `main`:

1. Create the PR from `develop` to `main` on GitHub
2. Title it something like "Release vX.Y.Z"
3. Once approved and merged, continue to tagging

### 8. Create Release Tag

After `develop` has been merged into `main`, create the release tag from `main`:

```bash
git checkout main
git pull origin main
env GIT_COMMITTER_DATE=$(git log -n1 --pretty=%aD) git tag -a -f -m "Release X.Y.Z" vX.Y.Z
```

**Important**: The tag must follow the format `v[0-9]+.[0-9]+.[0-9]+` to trigger
the CI workflows.

### 9. Push Tag

Push the tag to trigger the release workflows:

```bash
git push origin vX.Y.Z
```

## Automated Release Process

Once the tag is pushed, the following automated processes occur:

### GitHub Release Creation (.github/workflows/release.yaml)

This workflow:

1. Triggers on version tags matching `v[0-9]+.[0-9]+.[0-9]+`
2. Creates a versioned directory with Docker Compose files
3. Packages the files into both .zip and .tar.gz archives
4. Creates a **draft** GitHub release
5. Uploads the archives as release assets

**Note**: The release is created as a draft and must be manually published
through GitHub's UI.

### Docker Image Building (.github/workflows/docker.yaml)

This workflow:

1. Builds multi-architecture Docker images (linux/amd64 and linux/arm64)
2. Creates images for:
   - OpenMina node (`openmina/openmina`)
   - Frontend (`openmina/frontend`)
3. Tags images with:
   - Branch name (for branches)
   - Short SHA
   - Semantic version (for tags)
   - `latest` (for main branch)
   - `staging` (for develop branch)

## Post-Release Steps

### 1. Review Draft Release

1. Go to the GitHub releases page
2. Find the draft release created by the workflow
3. Review the release notes and assets
4. Ensure the release assets include:
   - `openmina-vX.Y.Z-docker-compose.zip`
   - `openmina-vX.Y.Z-docker-compose.tar.gz`
5. Add any additional release notes or documentation if needed

### 2. Publish Release

Click "Publish release" in GitHub's UI to make the release publicly available.

### 3. Verify Docker Images

Verify the Docker images are available on Docker Hub:

```bash
docker pull openmina/openmina:vX.Y.Z
docker pull openmina/frontend:vX.Y.Z
```

Check that both `amd64` and `arm64` architectures are available:

```bash
docker manifest inspect openmina/openmina:vX.Y.Z
```

## Version Tagging Best Practices

- Always use semantic versioning: `vMAJOR.MINOR.PATCH`
- Use annotated tags (`-a` flag) for releases
- Include a meaningful message with `-m`
- Preserve commit date with `GIT_COMMITTER_DATE` for traceability

## Troubleshooting

### CI Workflow Not Triggered

Ensure:

- The tag format matches exactly: `v[0-9]+.[0-9]+.[0-9]+`
- The tag was pushed to the remote repository
- Check GitHub Actions for any workflow errors

### Docker Build Failures

- Check the GitHub Actions logs for specific error messages
- Ensure all tests pass before creating a release
- Verify Dockerfile syntax and dependencies

## Example Release Commit Structure

A typical release PR (like #1134) includes these commits:

1. **chore: Update CHANGELOG** - Updates the changelog with release notes and
   comparison link
2. **chore: Bump version to X.X.X** - Result of running `versions.sh`
3. **chore: Update Cargo.lock** - Updated dependencies lock file
4. **chore: Update version in docker compose files** - Docker image version
   updates

## Internal/Patch Releases

For internal or non-public patch releases (e.g., vX.Y.Z+1), you can tag directly
from `develop`:

1. Follow steps 1-5 above (create release branch, update versions, commit
   changes, PR to develop)
2. After merging to `develop`, tag directly from `develop`:
   ```bash
   git checkout develop
   git pull origin develop
   env GIT_COMMITTER_DATE=$(git log -n1 --pretty=%aD) git tag -a -f -m "Release X.Y.Z+1" vX.Y.Z+1
   git push origin vX.Y.Z+1
   ```
3. The same CI/CD workflows will trigger and create draft releases

This approach is useful for:

- Quick fixes that need immediate deployment
- Internal testing releases
- Patch releases that don't warrant a full main branch update

## Reference

For examples of previous releases, see:

- PR #1134 (Release v0.16.0) and similar release PRs
- The git tag history: `git tag -l`
- The CHANGELOG.md file for release note formats
