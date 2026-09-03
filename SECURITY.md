# Security Policy

## Overview

DreamizDB is an open-source database systems project. Security, data integrity, privacy, and responsible disclosure are important considerations in the development and use of the project.

This document describes how to report potential security vulnerabilities in DreamizDB.

## Supported Versions

Security fixes are generally focused on the latest development version of DreamizDB.

| Version        | Security Support |
| -------------- | ---------------- |
| Latest `main`  | Supported        |
| Older versions | Best effort      |

Because DreamizDB is under active development, users should prefer the latest published version or commit when possible.

## Reporting a Vulnerability

If you believe you have discovered a security vulnerability in DreamizDB, please report it privately rather than opening a public GitHub issue.

Please provide:

* A clear description of the vulnerability.
* The affected component, file, module, or feature.
* The conditions required to reproduce the issue.
* Reproduction steps or a minimal proof of concept, where appropriate.
* The potential security impact.
* Any suggested mitigation or remediation, if known.

Please avoid including passwords, API keys, private credentials, personal information, production database contents, or other sensitive information in a vulnerability report.

## Private Disclosure

Security vulnerabilities should be disclosed privately so that they can be investigated and, where appropriate, addressed before public disclosure.

Please do not publicly disclose an exploitable vulnerability, proof of concept containing sensitive information, or detailed exploitation instructions before the issue has been reviewed.

After a vulnerability has been addressed, the maintainers may publish an appropriate security advisory or other disclosure describing the issue and its remediation.

## Security Issues in Dependencies

DreamizDB depends on third-party Rust crates and other software components.

Security issues discovered in dependencies should be reported according to the relevant dependency's security reporting process when the vulnerability originates in that dependency.

Dependency vulnerabilities that materially affect DreamizDB should also be considered in the project's security review.

## Data and Privacy

DreamizDB may be used to store or process application data.

Users and operators are responsible for:

* Protecting access credentials and secrets.
* Securing the systems on which DreamizDB runs.
* Controlling access to database files and runtime data.
* Applying appropriate filesystem and operating-system permissions.
* Protecting backups and exported database data.
* Avoiding the inclusion of confidential or personal information in public issue reports, logs, benchmarks, or repository contributions.

The public DreamizDB repository must not contain passwords, API keys, private keys, credentials, production database contents, or other confidential information.

## Security Scope

Security review may include, but is not limited to:

* Authentication and authorization mechanisms, where applicable.
* Data integrity and persistence.
* File and filesystem handling.
* Input validation.
* Memory and resource safety.
* Concurrency and synchronization.
* Network-facing components, where applicable.
* Dependency vulnerabilities.
* Information disclosure.
* Unsafe or unintended access to stored data.
* Build and release processes.

DreamizDB is an actively developed project. Features that are experimental, incomplete, or explicitly documented as such should not automatically be considered production-ready.

## Responsible Disclosure

We encourage responsible disclosure and good-faith security research.

Security researchers should make reasonable efforts to:

* Avoid accessing or modifying data that does not belong to them.
* Avoid disrupting services or systems.
* Avoid destructive testing.
* Protect any sensitive information encountered during research.
* Provide sufficient information to reproduce and investigate the reported issue.

## Security Updates

When a confirmed security issue is addressed, the project may update the relevant source code, documentation, release notes, GitHub security advisory, or other appropriate project documentation.

Users should keep their DreamizDB installation and dependencies reasonably up to date.

## Contact

For security reports, use the private security reporting mechanism provided by the DreamizDB GitHub repository.

Please do not use public GitHub issues for undisclosed security vulnerabilities.

---

**Note:** This security policy does not replace the Apache License 2.0 or create additional warranties, guarantees, or contractual obligations for DreamizDB or its contributors.
