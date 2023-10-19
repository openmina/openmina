# K8s Cluster Usage for Testing

## Daily Runs Namespace

The namespace `test-openmina-daily` is used. It has a service account
`github-tester` with `edit` role that allows it to have full control over the
namespace's resources. This account is used by GitHub actions that run daily
tests.

