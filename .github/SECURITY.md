# Security Policy

Ateneo is an institutional repository. Institutions trust it with their
documents and their users' accounts, so we take security seriously.

## Supported versions

Ateneo is in early development (0.x). Only the **latest released version**
receives security fixes. Once 1.0 is released, this table will list the
supported versions.

| Version        | Supported          |
| -------------- | ------------------ |
| Latest 0.x     | :white_check_mark: |
| Older versions | :x:                |

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub
issues, discussions, or pull requests.**

Report them privately using GitHub's private vulnerability reporting:

1. Go to the [Security tab](https://github.com/ateneo-rs/ateneo/security) of
   this repository.
2. Click **Report a vulnerability**.
3. Fill in the form.

If you cannot use GitHub, email **jamespnpm1624@outlook.com** with the
subject `[SECURITY] Ateneo`.

Please include, if possible:

- The affected version of Ateneo.
- A description of the vulnerability and its impact.
- Steps to reproduce it, or a proof of concept.
- Any suggested fix or mitigation.

You can write in **Spanish or English** (puedes escribir en español).

## What to expect

Ateneo is currently maintained by a single person, so please allow for
these timelines:

- **Acknowledgement:** within 7 days.
- **Initial assessment:** within 14 days, including whether we consider
  it a vulnerability and its severity.
- **Fix:** we aim to release a fix within 90 days of the report,
  depending on complexity. We will keep you informed of progress.

## Coordinated disclosure

We follow coordinated vulnerability disclosure:

- Please give us reasonable time to release a fix before disclosing the
  vulnerability publicly.
- Once a fix is released, we will publish a security advisory on GitHub
  and, when appropriate, request a CVE identifier.
- We will credit you in the advisory, unless you prefer to remain
  anonymous.

## Scope

In scope:

- The Ateneo server (`ateneo` crate) and its built-in admin panel.
- The npm package `ateneo` and its platform packages (`@ateneo-rs/*`).

Out of scope:

- Misconfiguration of a specific institution's server or network.
- Third-party services that Ateneo connects to (Google, Microsoft, S3,
  DataCite, etc.). Please report those to the respective provider.
- Denial of service through sheer volume of traffic.

## Safe harbor

We will not take legal action against anyone who reports a vulnerability
in good faith, follows this policy, avoids accessing or modifying other
people's data, and avoids degrading the service of any Ateneo installation.