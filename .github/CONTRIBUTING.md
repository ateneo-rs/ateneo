# Contributing to Ateneo

Thank you for your interest in contributing!
¡Gracias por tu interés en colaborar! Puedes escribir en español o en inglés.

## Developer Certificate of Origin (DCO)

Ateneo uses the [Developer Certificate of Origin](https://developercertificate.org/).
By adding a `Signed-off-by` line to your commits, you certify that you wrote
the code or otherwise have the right to submit it under the project's license
(AGPL-3.0-or-later).

Sign off every commit with the `-s` flag:

    git commit -s -m "feat: describe your change"

This adds a line like this to the commit message:

    Signed-off-by: Your Name <your.email@example.com>

Pull requests with unsigned commits cannot be merged.

### Forgot to sign off?

Fix the last commit:

    git commit --amend -s --no-edit
    git push --force-with-lease

## Commit messages

We use [Conventional Commits](https://www.conventionalcommits.org/):
`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.

## License headers

Every new source file must start with:

    // SPDX-FileCopyrightText: 2026 Your Name
    // SPDX-License-Identifier: AGPL-3.0-or-later

## Security

Never report security vulnerabilities in public issues.
See [SECURITY.md](SECURITY.md).

## Code of Conduct

By participating, you agree to follow our [Code of Conduct](CODE_OF_CONDUCT.md).