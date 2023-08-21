# Openmina Images

Currently CI builds container images when pushing to the
`feat/standalone_snark_worker` or to a PR targeted to that branch.

Images are available at the Docker Hub repository `openmina/openmina`.

For the branch itself, two tags produced, one derived from the commit hash
(first 8 chars), and another one is `latest`.

For a PR from a branch named `some/branch`, two tags are produced, one is `some-branch-<commit-hash>` and another one is `some-branch-latest`.

