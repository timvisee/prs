[![Build status on GitLab CI][gitlab-ci-master-badge]][gitlab-ci-link]
[![Newest release on crates.io][crate-version-badge]][crate-link]
[![Project license][crate-license-badge]](LICENSE)

[crate-license-badge]: https://img.shields.io/crates/l/prs.svg
[crate-link]: https://crates.io/crates/prs
[crate-version-badge]: https://img.shields.io/crates/v/prs.svg
[gitlab-ci-link]: https://gitlab.com/timvisee/prs/pipelines
[gitlab-ci-master-badge]: https://gitlab.com/timvisee/prs/badges/master/pipeline.svg

# prs

> A fast, secure & convenient password manager using GPG and git for
> synchronisation.

`prs` is a fast, secure and convenient password manager for the terminal.
It features [`gpg`][gpg] to securely store your details and integrates
[`git`][git] for synchronisation between multiple machines and history tracking.

`prs` is heavily inspired by [`pass`][pass] and uses the same file structure
with some additions. `pr` therefore works together with `pass` and all other
compatible clients, extensions and migration scripts.

## Features
- Fully featured fast & friendly command line tool
- Temporary copy secrets to clipboard
- Uses the battle-tested [`gpg`][gpg] to secure your secrets
- Automatic synchronisation with [`git`][git] including history tracking
- Supports multiple machines with easy recipient management
- Compatible with [`pass`][pass]
- Easily scriptable with `-y`, `-f`, `-I` flags.
- Accurate & useful error reporting

## Requirements
- Linux, macOS, Windows, FreeBSD, Android (other BSDs might work)
- A terminal :cool:
- `gpg` and `git`
  - Ubuntu, Debian and derivatives: `apt install git gpg`
  - CentOS/Red Hat/openSUSE/Fedora: `apt install git gnupg`
  - Arch: `pacman -S git gnupg`

## Security
`prs`'s security is backed by [`gpg`][gpg] which is used all over the world and
has been battle-tested for more than 20 years.

`prs` is secure to keep your deepest secrets when assuming the following:

- You keep the password store directory (`~/.password-store`) safe
- When using sync with `git`: you keep your remote repository safe
- You use secure GPG keys
- Your machine is secure

Note: `prs` tool does not provide any warranty in any way, shape or form for
leaked secrets.

## Help
```
$ prs help

prs-cli 0.1.0
Tim Visee <3a4fb3964f@sinenomine.email>
Fast, secure & convenient password manager with GPG & git

USAGE:
    prs [FLAGS] <SUBCOMMAND>

FLAGS:
    -f, --force          Force the action, ignore warnings
    -h, --help           Prints help information
    -I, --no-interact    Not interactive, do not prompt
    -q, --quiet          Produce output suitable for logging and automation
    -V, --version        Prints version information
    -v, --verbose        Enable verbose information and logging
    -y, --yes            Assume yes for prompts

SUBCOMMANDS:
    add             Add a secret
    copy            Copy secret to clipboard
    duplicate       Duplicate a secret
    edit            Edit a secret
    generate        Generate a secure secret
    git             Invoke git command in password store
    help            Prints this message or the help of the given subcommand(s)
    housekeeping    Housekeeping utilities
    init            Initialize new password store
    list            List all secrets
    move            Move a secret
    recipients      Manage store recipients
    remove          Remove a secret
    show            Display a secret
    sync            Sync password store
```

## License
This project is released under the GNU GPL-3.0 license.
Check out the [LICENSE](LICENSE) file for more information.

The library portion of this project is licensed under the MIT license.
Check out the [lib/LICENSE](lib/LICENSE) file for more information.

[git]: https://git-scm.com/
[gpg]: https://gnupg.org/
[pass]: https://www.passwordstore.org/
