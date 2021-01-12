[![Build status on GitLab CI][gitlab-ci-master-badge]][gitlab-ci-link]
[![Newest release on crates.io][crate-version-badge]][crate-link]
[![Project license][crate-license-badge]](LICENSE)

[crate-license-badge]: https://img.shields.io/crates/l/prs-lib.svg
[crate-link]: https://crates.io/crates/prs-cli
[crate-version-badge]: https://img.shields.io/crates/v/prs-lib.svg
[gitlab-ci-link]: https://gitlab.com/timvisee/prs/pipelines
[gitlab-ci-master-badge]: https://gitlab.com/timvisee/prs/badges/master/pipeline.svg

# prs

> A secure, fast & convenient password manager CLI using GPG and git to sync.

`prs` is a secure, fast and convenient password manager for the terminal.
It features [GPG][gpg] to securely store your secrets and integrates
[`git`][git] for automatic synchronization between multiple machines. It also
features a built-in password generator, recipient management, history tracking,
rollbacks, housekeeping utilities and more.

[![prs usage demo][usage-demo-svg]][usage-demo-asciinema]  
_No demo visible here? View it on [asciinema][usage-demo-asciinema]._

`prs` is heavily inspired by [`pass`][pass] and uses the same file structure
with some additions. `prs` therefore works alongside with `pass` and all other
compatible clients, extensions and migration scripts.

- [Features](#features)
- [Usage](#usage)
- [Requirements](#requirements)
- [Install](#install)
- [Security](#security)
- [FAQ](#faq)
- [Help](#help)
- [License](#license)

## Features
- Fully featured fast & friendly command line tool
- Temporary copy secrets to clipboard
- Uses the battle-tested [GPG][gpg] to secure your secrets
- Automatic synchronization with [`git`][git] including history tracking
- Supports multiple machines with easy recipient management
- Easily edit secrets using your default editor
- Supports smart aliases, property selection
- Compatible with [`pass`][pass]
- Scriptable with `-y`, `-f`, `-I` flags
- Accurate & useful error reporting

`prs` includes some awesome tweaks and optimizations:

- Greatly improved synchronisation speed through `git` with connection reuse
- Super fast interactive secret/recipient selection through [`skim`][skim]
- Prevents messing with your clipboard with unexpected overwrites
- Commands have short and conventional aliases for faster and more convenient usage
- Uses security best practices (secrets: zeroed, `mlock`, `madvice`, no format, [etc](#security))

## Usage
```bash
# Show useful commands (based on current password store state)
prs

# Easily add, modify and remove secrets with your default editor:
prs add site/gitlab.com
prs edit site/gitlab.com
prs duplicate my/secret extra/secret
prs alias my/secret extra/alias
prs move my/secret extra/secret
prs remove site/gitlab.com

# Or generate a new secure password
prs generate site/gitlab.com

# Temporary show or copy secrets to clipboard:
prs show
prs show site/gitlab.com
prs copy
prs copy site/gitlab.com

# Manually synchronize password store with remote repository or do some housekeeping
prs sync
prs housekeeping
prs housekeeping run
prs housekeeping recrypt

# Manage recipients when using multiple machines
prs recipients add
prs recipients list
prs recipients remove
prs recipients generate
prs recipients export

# Commands support shorter/conventional commands and aliases
prs a secret  # add
prs c         # copy
prs s         # show
prs rm        # remove
prs yeet      # remove

# List all commands and help
prs help
```

## Requirements
- Linux, macOS, FreeBSD, Android (other BSDs might work)
- A terminal :sunglasses:
- `gpg`, `gpgme` and `git`
  - Ubuntu, Debian and derivatives: `apt install git gpg libgpgme11`
  - CentOS/Red Hat/openSUSE/Fedora: `yum install git gnupg gpgme`
  - Arch: `pacman -S git gnupg gpgme`
  - Alpine: `apk add git gnupg gpgme`
  - macOS: `brew install gpg gpgme` (+ `gtk+3`)

## Install
Because `prs` is still in early stages, only limited installation options are
available right now. Feel free to contribute.

Make sure you meet and install the [requirements](#requirements).

Find the latest binaries on the latest release page:
- [GitHub][github-release-latest]
- [GitLab][gitlab-releases]
- [GitLab package registry][gitlab-packages] for `prs`

_Note: for Linux the GNU (not musl) binary is recommended if it works, because it
has better clipboard/notification support._

## Security
Security is backed by [`gpg`][gpg] which is used all over the world and
has been battle-tested for more than 20 years.

In summary, `prs` is secure to keep your deepest secrets when assuming the following:

- You keep the password store directory (`~/.password-store`) safe
- When using sync with `git`: you keep your remote repository safe
- You use secure GPG keys
- Your machines are secure

The content of secrets is encrypted and secured. Secrets are stored as encrypted
GPG files. Some metadata is visible without decryption however.
The name of a secret (file name), modification time (file modification time) and
encrypted size (file size) are visible when you have access to the password
store directory.

Security best practices are used in `prs` to prevent accidentally leaking
any secret data. Sensitive data such as plaintext, ciphertext and others are
referred to as 'secret' here.

Secrets are/use:

- Zeroed on drop
- Locked to physical memory, cannot leak to swap/disk ([`mlock`][security-mlock])
- Locked into memory, cannot be dumped/not included in core ([`madvice`][security-mlock])
- Not written to disk to edit (if possible)
- String formatting is blocked
- Constant time comparison to prevent time based attacks
- Minimally cloned

[security-mlock]: https://man7.org/linux/man-pages/man2/mlock.2.html
[security-madvice]: https://man7.org/linux/man-pages/man2/madvise.2.html

The protection against leaking secrets has its boundaries, notably:

- `prs show` prints secret data to stdout
- `prs edit` may store secrets in a secure temporary file on disk if secure
  locations such as (`/dev/shm`) are not available, it then opens it in your
  default editor, and removes it afterwards
- `prs copy` copies secret data to your clipboard, and clears it after 20
  seconds

[![Security](./res/xkcd_538.png)][xkcd538]  
_Reference: [XKCD 538][xkcd538]_

Note: `prs` does not provide any warranty in any way, shape or form for damage
due to leaked secrets or other issues.

## FAQ
#### Is `prs` secure? How secure is `prs`?
Please read the [Security](#security) section.

#### How do I use sync with git?
If you already have a remote password store repository that is
[compatible](#is-prs-compatible-with-pass) with `prs`, clone it using:

```bash
# Clone existing remote password store, automatically enables sync
prs clone MY_GIT_URL

# List secrets
prs list
```

If you do not have a remote password store repository yet, create one (an empty
private repository on GitHub or GitLab for example), and run the following:

```bash
# Initialize new password store (if you haven't done so yet)
prs init

# Initialize sync functionality (if you haven't done so yet)
prs sync init

# Set your remote repository URL and sync to push your password store
prs sync remote MY_GIT_URL
prs sync
```

When sync is enabled on your password store, all commands that modify your
secrets will automatically keep your remote store in sync.

To manually trigger a sync because you edited a secret on a different machine,
run:

```bash
prs sync
```

#### How do I use `prs` on multiple machines and sync between them?
_Note: adding and using your existing password store on a new/additional machine
requires you to have access to a machine that already uses the store during setup._

First, you must have a password store on one machine. Create one (with `prs init`)
if you don't have any yet.
You must set up sync with a remote git repository for this passwords store, see
the [How do I use sync with git](#how-do-i-enable-sync-with-git) section.

To use your existing password store on a new machine, install `prs` and clone
your remote password store:

```bash
# On new machine: clone existing password store from git remote
prs clone MY_GIT_URL
```

Then add a recipient to the password store for your new machine. I highly
recommend to use a new recipient (GPG key pair) for each machine (so you won't
have to share secret GPG keys). Add an existing secret GPG key as recipient, or
generate a new GPG key pair, using:

```bash
# On new machine: add existing recipient or generate new one
prs recipients add --secret
# or
prs recipients generate
```

Your new machine can't read any password store secrets yet, because they are not
encrypted for its recipient yet. Go back to an existing machine you already use
the store on, and re-encrypt all secrets to also encrypt them for the new
recipient:

```bash
# On existing machine: re-encrypt all secrets
prs housekeeping recrypt --all
```

This may take a while. Once done, sync on your new machine to pull in the
updated secrets:

```bash
# On new machine: pull in all re-crypted secrets
prs sync

# You're done!
prs list
```

#### How do I use `prs` on mobile?
`prs` itself does not support mobile, but there are compatible clients you can
use to use your password store on mobile.

See [Compatible Clients][pass-compatible-clients] on `pass`s website.

#### Can I recover my secrets if I lost my key?
No, if you lose all keys, there is no way to recover your secrets.

You might lose your key (recipient, GPG secret key) if your machine crashes or
if you reinstall it's operating system.

If you are using the same password store on multiple machines with git sync, you
can still read the secrets on your other machines. To re-add the machine you
lost your key on, remove the password store from it and see
[this](#how-do-i-use-prs-on-multiple-machines-and-sync-between-them) section.

#### Is `prs` compatible with `pass`?
Yes.

`prs` uses the same file structure as [`pass`][pass]. Other `pass` clients
should be able to view and edit your secrets.

`prs` does add additional files and settings, some `prs` features might not work
with other `pass` clients.

See a list of compatible `pass` clients [here][pass-compatible-clients].

## Help
```
$ prs help

prs-cli 0.1.5
Tim Visee <3a4fb3964f@sinenomine.email>
Secure, fast & convenient password manager CLI with GPG & git sync

USAGE:
    prs [FLAGS] [SUBCOMMAND]

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
    alias           Alias/symlink a secret
    clone           Clone existing password store
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

The library portion of this project is licensed under the GNU LGPL-3.0 license.
Check out the [lib/LICENSE](lib/LICENSE) file for more information.

[git]: https://git-scm.com/
[github-release-latest]: https://github.com/timvisee/prs/releases/latest
[gitlab-packages]: https://gitlab.com/timvisee/prs/-/packages
[gitlab-releases]: https://gitlab.com/timvisee/prs/-/releases
[gpg]: https://gnupg.org/
[pass-compatible-clients]: https://www.passwordstore.org#other
[pass]: https://www.passwordstore.org/
[skim]: https://github.com/lotabout/skim
[usage-demo-asciinema]: https://asciinema.org/a/368611
[usage-demo-svg]: ./res/demo.svg
[xkcd538]: https://xkcd.com/538/
