[![Build status on GitLab CI][gitlab-ci-master-badge]][gitlab-ci-link]
[![Newest release on crates.io][crate-version-badge]][crate-link]
[![Project license][crate-license-badge]](LICENSE)

[crate-license-badge]: https://img.shields.io/crates/l/prs.svg
[crate-link]: https://crates.io/crates/prs
[crate-version-badge]: https://img.shields.io/crates/v/prs.svg
[gitlab-ci-link]: https://gitlab.com/timvisee/prs/pipelines
[gitlab-ci-master-badge]: https://gitlab.com/timvisee/prs/badges/master/pipeline.svg

# prs

> A secure, fast & convenient password manager using GPG and git for
> synchronisation.

`prs` is a secure, fast and convenient password manager for the terminal.
It features [GPG][gpg] to securely store your details and integrates
[`git`][git] for automatic synchronisation between multiple machines. It also
features a built-in password generator, recipient management, history tracking,
rollbacks, housekeeping utilities and more.

`prs` is heavily inspired by [`pass`][pass] and uses the same file structure
with some additions. `prs` therefore works alongside with `pass` and all other
compatible clients, extensions and migration scripts.

- [Features](#features)
- [Usage](#usage)
- [Requirements](#requirements)
- [Security](#security)
- [Help](#help)
- [License](#license)

## Features
- Fully featured fast & friendly command line tool
- Temporary copy secrets to clipboard
- Uses the battle-tested [GPG][gpg] to secure your secrets
- Automatic synchronisation with [`git`][git] including history tracking
- Supports multiple machines with easy recipient management
- Easily edit secrets using your default editor
- Compatible with [`pass`][pass]
- Scriptable with `-y`, `-f`, `-I` flags
- Accurate & useful error reporting

`prs` includes some awesome tweaks and optimizations:

- Greatly improved synchronisation speed through `git` with connection reuse
- Super fast secret/recipient selection through [`skim`][skim]
- Prevents messing with your clipboard with unexpected overwrites
- Commands have short and conventional aliases for faster and more convenient usage

## Usage
```bash
# Easily add, modify and remove secrets with your default editor:
prs add site/gitlab.com
prs edit site/gitlab.com
prs duplicate my/secret extra/secret
prs move my/secret extra/secret
prs remove site/gitlab.com

# Or generate a new secure password
prs generate site/gitlab.com

# Temporary show or copy secrets to clipboard:
prs show
prs show site/gitlab.com
prs copy
prs copy site/gitlab.com

# Manually synchronise password store with remote repository or do some housekeeping
prs sync
prs housekeeping recrypt
prs housekeeping sync-keys

# Manage recipients when using multiple machines
prs recipients add
prs recipients list
prs recipients remove
prs recipients generate
prs recipients export

# All commands support shorter/conventional commands and aliases
prs a secret  # add
prs c         # copy
prs s         # show
prs rm        # remove
prs yeet      # remove
```

## Requirements
- Linux, macOS, Windows, FreeBSD, Android (other BSDs might work)
- A terminal :sunglasses:
- `gpg` and `git`
  - Ubuntu, Debian and derivatives: `apt install git gpg`
  - CentOS/Red Hat/openSUSE/Fedora: `apt install git gnupg`
  - Arch: `pacman -S git gnupg`

## Security
Security is backed by [`gpg`][gpg] which is used all over the world and
has been battle-tested for more than 20 years.

`prs` is secure to keep your deepest secrets when assuming the following:

- You keep the password store directory (`~/.password-store`) safe
- When using sync with `git`: you keep your remote repository safe
- You use secure GPG keys
- Your machines are secure

Note: `prs` does not provide any warranty in any way, shape or form for damage
due to leaked secrets or other issues.

## Help
```
$ prs help

prs-cli 0.1.0
Tim Visee <3a4fb3964f@sinenomine.email>
Secure, fast & convenient password manager with GPG & git

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
[skim]: https://github.com/lotabout/skim
