# Security Policy

Canonical repository path: `.github/SECURITY.md`.

## Reporting A Vulnerability

Report vulnerabilities privately through
[GitHub Security Advisories](https://github.com/atxinbao/NTPRO/security/advisories/new).
Alternatively, email `security@nautechsystems.io`.

Include the affected commit or release, reproduction steps, impact, and any
suggested remediation. Do not disclose the issue publicly before a coordinated
fix is available and do not access data beyond what is required to demonstrate
the issue.

## Supported Version

NTPRO supports the current Rust-only release baseline. Older versions may
contain issues already corrected in the current source tree.

## Repository Controls

- Rust dependencies are locked by `Cargo.lock` and audited by cargo-audit,
  cargo-deny, cargo-vet, and OSV Scanner.
- GitHub Actions workflows are scanned by Zizmor and third-party actions are
  pinned to immutable commit SHAs.
- Critical manifests, workflows, policies, and release files are protected by
  CODEOWNERS and branch rules.
- Rust formatting, clippy, tests, golden traces, current governance, and the
  v0.32.0 backend freeze contract run before release changes are accepted.
- No Python package, wheel, sdist, or container publication path is current
  repository authority.

## Release Controls

The v0.32.0 tag and GitHub Release are the frozen backend baseline. Their tag
commit, hosted release gate, release body, and source-controlled freeze
registry must remain consistent. A later release or capability family requires
separately scoped governance and does not inherit mutation, adapter send, live
exchange, retry, remediation, or trading-control capability.

The current release can be verified with:

```bash
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/check_github_release_published.sh
scripts/ai/verify_release.sh current-governance backend-freeze-baseline
```

## Dependency Risk

Known transitive Rust advisories without an available fix are documented in
the repository audit policy with scope and rationale. New advisories are
reported by scheduled security automation and assessed against runtime reach,
severity, and available upstream remediation.
