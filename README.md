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
rollbacks, housekeeping utilities, Tomb support and more.

[![prs usage demo][usage-demo-svg]][usage-demo-asciinema]  
_No demo visible here? View it on [asciinema][usage-demo-asciinema]._

`prs` is heavily inspired by [`pass`][pass] and uses the same file structure
with some additions. `prs` therefore works alongside with `pass` and all other
compatible clients, extensions and migration scripts.

- [Features](#features)
- [Usage](#usage)
- [Requirements](#requirements)
- [Install](#install)
- [Build](#build)
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
- Compatible with [`pass`][pass][*](#is-prs-compatible-with-pass)
- Supports Linux, macOS, Windows, FreeBSD and others, supports X11 and Wayland
- Supports multiple cryptography backends (more backends & crypto in the future)
- Seamless [Tomb][tomb] support to prevent metadata leakage[*](#what-is-tomb)
- Support for TOTP tokens for two-factor authentication
- Scriptable with `-y`, `-f`, `-I` flags
- Accurate & useful error reporting

`prs` includes some awesome tweaks and optimizations:

- Greatly improved synchronisation speed through `git` with connection reuse[*](./docs/connection-reuse.md)
- Super fast interactive secret/recipient selection through [`skim`][skim]
- Prevents messing with your clipboard, no unexpected overwrites or clipboard loss
- When using Tomb, it is automatically opened, closed and resized for you
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
- Linux, macOS, FreeBSD, Windows (other BSDs might work)
- A terminal :sunglasses:
- _And:_

#### Recommended
- Run: _`git`, `gnupg`, `gpgme`_
  - Ubuntu, Debian and derivatives: `apt install git gpg libgpgme11 tomb`
  - CentOS/Red Hat/openSUSE/Fedora: `yum install git gnupg gpgme`
  - Arch: `pacman -S git gnupg gpgme tomb`
  - Alpine: `apk add git gnupg gpgme`
  - macOS: `brew install gnupg gpgme`
  - Windows: `scoop install git gpg fzf`
- Build: _`git`, `gnupg`, `gpgme` dev packages and dev utilities_
  - Ubuntu, Debian and derivatives: `apt install git gpg build-essential pkg-config python3 xorg-dev libx11-xcb-dev libdbus-1-dev libgpgme-dev tomb`
  - CentOS/Red Hat/openSUSE/Fedora: `yum install git gnupg gpgme-devel pkgconfig python3 xorg-x11-devel libxcb-devel`
  - Arch: `pacman -S git gnupg gpgme pkgconf python3 xorg-server libxcb tomb`
  - Alpine: `apk add git gnupg gpgme-dev pkgconfig`
  - macOS: `brew install gnupg gpgme`
  - Windows: `scoop install git gpg fzf`

#### Specific
Specific features or crates require specific dependencies as shown below.

The listed dependencies might be incorrect or incomplete. If you believe there
to be an error, please feel free to contribute.

<details>
  <summary>[Required] Minimal requirements</summary>

  - Run & build: _`gpg` and `git`_
    - Ubuntu, Debian and derivatives: `apt install git gpg`
    - CentOS/Red Hat/openSUSE/Fedora: `yum install git gnupg`
    - Arch: `pacman -S git gnupg`
    - Alpine: `apk add git gnupg`
    - macOS: `brew install gpg`
    - Windows: `scoop install git gpg fzf`
</details>

<details>
  <summary>[Recommended] Feature: GPGME backend</summary>

  _`--feature=backend-gpgme`_

  - Run: _`gpgme` & build tools_
    - Ubuntu, Debian and derivatives: `apt install libgpgme11`
    - CentOS/Red Hat/openSUSE/Fedora: `yum install gpgme`
    - Arch: `pacman -S gpgme`
    - Alpine: `apk add gpgme`
    - macOS: `brew install gpgme`
    - Windows: _not supported_
  - Build: _`gpgme` dev package
    - Ubuntu, Debian and derivatives: `apt install build-essential pkg-config libgpgme-dev`
    - CentOS/Red Hat/openSUSE/Fedora: `yum install pkgconfig gpgme-devel`
    - Arch: `pacman -S pkgconf gpgme`
    - Alpine: `apk add pkgconfig gpgme-dev`
    - macOS: `brew install gpgme`
    - Windows: _not supported_
</details>

<details>
  <summary>[Recommended] Feature: Clipboard</summary>

  _`--feature=clipboard`_

  - Run:
    - Ubuntu, Debian and derivatives: `apt install xorg libx11-xcb-dev wl-clipboard`
    - CentOS/Red Hat/openSUSE/Fedora: `yum install pkgconfig xorg libxcb wl-clipboard`
    - Arch: `pacman -S pkgconf xorg-server python3 libxcb wl-clipboard`
    - Alpine: _?_
    - macOS: _none_
    - Windows: _none_
  - Build:
    - Ubuntu, Debian and derivatives: `apt install build-essential pkg-config python3 xorg-dev libx11-xcb-dev`
    - CentOS/Red Hat/openSUSE/Fedora: `yum install pkgconfig python3 xorg-x11-devel libxcb-devel`
    - Arch: `pacman -S pkgconf xorg-server python3 libxcb`
    - Alpine: _?_
    - macOS: _none_
    - Windows: _none_

  Note: `xorg`, `libx11-xcb` are only required at runtime when using X11.
  `wl-clipboard` are only required at runtime when using Wayland.
</details>

<details>
  <summary>[Recommended] Feature: Notifications</summary>

  _`--feature=notify`_

  - Run:
    - Ubuntu, Debian and derivatives: _[something][linux-notifications] supporting notifications with libnotify_
    - CentOS/Red Hat/openSUSE/Fedora: _[something][linux-notifications] supporting notifications with libnotify_
    - Arch: _[something][linux-notifications] supporting notifications with libnotify_
    - Alpine: _[something][linux-notifications] supporting notifications with libnotify_
    - macOS: _none_
    - Windows: _none_
  - Build: _`gpgme` dev package_
    - Ubuntu, Debian and derivatives: `apt install libdbus-1-dev`
    - CentOS/Red Hat/openSUSE/Fedora: `yum install dbus-libs`
    - Arch: `pacman -S dbus`
    - Alpine: `apk add dbus`
    - macOS: _none_
    - Windows: _none_
</details>

<details>
  <summary>Feature: Tomb</summary>

  _`--feature=tomb`_

  - Run: `tomb`
    - Ubuntu, Debian and derivatives: `apt install tomb`
    - CentOS/Red Hat/openSUSE/Fedora: [installation][tomb-install]
    - Arch: `pacman -S tomb`
    - Alpine: [installation][tomb-install]
    - macOS: _not supported_
    - Windows: _not supported_
</details>

<details>
  <summary>Client: GTK3 client</summary>

  _crate: `prs-gtk3` @ [`./gtk3`](./gtk3)_

  - Run: _`gtk3`_
    - Ubuntu, Debian and derivatives: `apt install libgtk-3-0 libgl1-mesa0`
    - CentOS/Red Hat/openSUSE/Fedora: `yum install gtk3`
    - Arch: `pacman -S gtk3`
    - Alpine: `apk add gtk+3.0 --repository=http://dl-cdn.alpinelinux.org/alpine/edge/main`
    - macOS: `brew install gtk+3`
    - Windows: _not supported_
  - Build: _`gtk3` dev packages_
    - Ubuntu, Debian and derivatives: `apt install libgtk-3-dev libgl1-mesa-dev`
    - CentOS/Red Hat/openSUSE/Fedora: `yum install gtk3-devel`
    - Arch: `pacman -S gtk3`
    - Alpine: `apk add gtk+3.0-dev --repository=http://dl-cdn.alpinelinux.org/alpine/edge/main`
    - macOS: `brew install gtk+3`
    - Windows: _not supported_
</details>

## Install
Because `prs` is still in early stages, only limited installation options are
available right now. Feel free to contribute.

Make sure you meet and install the 'Run' [requirements](#requirements).

See the operating system/distribution specific instructions below:
- [Linux](#linux-all-distributions)
- [macOS](#macos)
- [Windows](#windows)
- [Other](#other)

### Linux (all distributions)
Limited installation options are currently available. See the list below.
Alternatively you may install it manually using the
[prebuilt binaries](#linux-prebuilt-binaries).

Only 64-bit (`x86_64`) packages and binaries are provided.
For other architectures and configurations you may [compile from source](#build).

More packages options will be coming soon.

#### Linux: Arch AUR packages

[» `prs`](https://aur.archlinux.org/packages/prs/) (compiles from source, latest release)  
[» `prs-git`](https://aur.archlinux.org/packages/prs-git/) (compiles from source, latest `master` commit)

```bash
yay -S prs
# or
aurto add prs
sudo pacman -S prs
# or using any other AUR helper

prs --help
```

#### Linux: Prebuilt binaries
Check out the [latest release][github-latest-release] assets for Linux binaries.  
Use the `prs-v*-linux-x64-static` binary, to minimize the chance for issues.
If it isn't available yet, you may use an artifact from a
[previous version][github-releases] instead, until it is available.

Make sure you meet and install the [requirements](#requirements) before you
continue.

You must make the binary executable, and may want to move it into
`/usr/local/bin` to make it easily executable:

```bash
# Rename binary to prs
mv ./prs-* ./prs

# Mark binary as executable
chmod a+x ./prs

# Move binary into path, to make it easily usable
sudo mv ./prs /usr/local/bin/

prs
```

### macOS
`prs` can be installed using [homebrew].  
Alternatively you may [compile from source](#build).

Make sure you've [`homebrew`][homebrew-install] installed, then run:

```bash
brew install prs
prs
```

_Note: this package isn't automatically updated on release, feel free to
contribute
[here](https://github.com/Homebrew/homebrew-core/blob/master/Formula/prs.rb)._

### Windows
Using the [`scoop` package](#windows-scoop-package) is recommended.  
Alternatively you may install it manually using the
[prebuilt binaries](#windows-prebuilt-binaries).

If you're using the [Windows Subsystem for Linux][wsl], it's highly recommended
to install the [prebuilt Linux binary](#prebuilt-binaries-for-linux) instead.

Only 64-bit (`x86_64`) binaries are provided.
For other architectures and configurations you may [compile from source](#build).

#### Windows: scoop package
Make sure you've [`scoop`][scoop-install] installed, then run:

```bash
scoop install prs
prs
```

#### Windows: Prebuilt binaries
Check out the [latest release][github-latest-release] assets for Windows binaries.
Use the `prs-v*-windows` binary.
If it isn't available yet, you may use an artifact from a
[previous version][github-releases] instead, until it is available.

You can use `prs` from the command line in the same directory:
```cmd
.\prs.exe
```

To make it globally invokable as `prs`, you must make the binary available in
your systems `PATH`.

#### Other

Find the latest binaries on the latest release page:

- [GitHub][github-release-latest]
- [GitLab][gitlab-releases]
- [GitLab package registry][gitlab-packages] for `prs`

_Note: for Linux the GNU (not musl) binary is recommended if it works, because it
has better clipboard/notification support._

```bash
# download binary from any source above

# make executable
chmod a+x ./prs

# optional: make globally executable
mv ./prs /usr/local/bin/prs

./prs --help
```

## Build

To build and install `prs` yourself, you need the following:

- Rust 1.65 or newer (MSRV)
- The 'Build' [requirements](#requirements).

_Not all features are supported on macOS or Windows. The default configuration
should work. When changing compile time features, make sure to check for
compatibility. See [compiler features](#compile-features--use-flags)._

### Compile and install
To compile and install `prs` with the default features follow these steps:

- Compile and install it directly from cargo:

  ```bash
  # Compile and install from cargo
  cargo install prs-cli -f

  # Start using prs
  prs --help
  ```

- Or clone the repository and install it with `cargo`:

  ```bash
  # Clone the project
  git clone https://github.com/timvisee/prs.git
  cd prs

  # Compile and install
  cargo install --path cli -f

  # Start using prs
  prs --help

  # or run it directly from cargo
  cargo run --release --package prs-cli -- --help

  # or invoke release binary directly
  ./target/release/prs --help
  ```

### Compile features / use flags

Different use flags are available for `prs` to toggle whether to include various
features and cryptography backends. The following features are available, some
of which are enabled by default:

| Feature             | In                    | Enabled | Description                                                |
| :-----------------: | :-------------------: | :-----: | :--------------------------------------------------------- |
| `alias`             | `prc-cli`             | Default | Support for secret aliases (partially supported on Windows)|
| `clipboard`         | `prs-cli`             | Default | Clipboard support: copy secret to clipboard                |
| `notify`            | `prs-cli`, `prs-gtk3` | Default | Notification support: notify on clipboard clear            |
| `tomb`              | _all_                 | Default | Tomb support for password store (only supported on Linux)  |
| `totp`              |`prs-cli`              | Default | TOTP token support for 2FA                                 |
| `backend-gpgme`     | _all_                 |         | GPG crypto backend using GPGME (not supported on Windows)  |
| `backend-gnupg-bin` | _all_                 | Default | GPG crypto backend using GnuPG binary                      |
| `select-skim`       | `prc-cli`             | Default | Interactive selection with skim (ignored on Windows)       |
| `select-skim-bin`   | `prs-cli`             |         | Interactive selection through external `skim` binary       |
| `select-fzf-bin`    | `prs-cli`             | Default | Interactive selection through external `fzf` binary        |

To enable features during building or installation, specify them with
`--features <features...>` when using `cargo`.
You may want to disable default features first using
`--no-default-features`.
Here are some examples:

```bash
# Default set of features with backend-gnupg-bin, install or build, one of
cargo install --path cli --features backend-gnupg-bin
cargo build --path cli --release --features backend-gnupg-bin

# No default features, except required, one of
cargo install --path cli --no-default-features --features backend-gpgme
cargo install --path cli --no-default-features --features backend-gnupg-bin

# With alias, clipboard and notification support, one of
cargo install --path cli --no-default-features --features backend-gpgme,alias,clipboard,notify
cargo install --path cli --no-default-features --features backend-gnupg-bin,alias,clipboard,notify
```

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
store directory. To protect against this metadata leakage you may use a
[Tomb][tomb-faq].

Security best practices are used in `prs` to prevent accidentally leaking
any secret data. Sensitive data such as plaintext, ciphertext and others are
referred to as 'secret' here.

Secrets are/use:

- Zeroed on drop
- Locked to physical memory, cannot leak to swap/disk ([`mlock`][security-mlock])
- Locked into memory, cannot be dumped/not included in core ([`madvice`][security-madvice])
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
if you reinstall its operating system.

If you are using the same password store on multiple machines with git sync, you
can still read the secrets on your other machines. To re-add the machine you
lost your key on, remove the password store from it and see
[this](#how-do-i-use-prs-on-multiple-machines-and-sync-between-them) section.

#### What is Tomb?
[Tomb][tomb] is a file encryption system. It can be used with `prs` to protect
against metadata leakage of your password store.

When using Tomb with `prs`, your password store is stored inside an encrypted
file. `prs` automatically opens and closes your password store Tomb for you as
needed. This makes it significantly harder for malicious programs to list your
password store contents.

This feature is inspired by
[`pass-tomb`](https://github.com/roddhjav/pass-tomb), which is a `pass`
extension for Tomb support. In `prs` this functionality is built-in.

_Note: Tomb is only supported on Linux._

#### How to use Tomb?
`prs` has built-in support for [Tomb][tomb] on Linux systems. Please make sure
`prs` is compiled with the `tomb` [feature](#compile-features--use-flags), and
that Tomb is installed.

To initialize a Tomb for your current password store, simply invoke:

```bash
# Initialize tomb, this may take some time
prs tomb init

# Read tomb status
prs tomb status
```

To initialize a new password store in a Tomb, first initialize the password
store then initialize the Tomb:

```bash
# Initialize new password store
prs init

# ...

# Initialize tomb, this may take some time
prs tomb init
```

If you already have a Tomb created with `pass-tomb`, no action is required.
`prs` has seamless support for it, and it should automatically manage it for
you. Invoke `prs tomb status` to confirm it is detected.

#### How to use Tomb on multiple machines?
A Tomb is local on your machine and is not synced. To use a Tomb on
multiple machines you must initialize it on each of them.

Simply run `prs tomb init` on machines you don't use a Tomb on yet, and after
cloning your password store on a new machine.

#### Is `prs` compatible with `pass`?
Yes.

`prs` uses the same file structure as [`pass`][pass]. Other `pass` clients
should be able to view and edit your secrets.

`prs` does add additional files and settings, some `prs` features may not work
with other `pass` clients.

While the backing file structure is compatible, the command-line interface is
not and differs from `pass`. This is to remove ambiguity and to improve overall
usability.

See a list of compatible `pass` clients [here][pass-compatible-clients].

## Help
```
$ prs help

prs 0.5.2
Tim Visee <3a4fb3964f@sinenomine.email>
Secure, fast & convenient password manager CLI with GPG & git sync

Usage: prs [OPTIONS] [COMMAND]

Commands:
  show          Display a secret
  copy          Copy secret to clipboard
  generate      Generate a secure secret
  add           Add a secret
  edit          Edit a secret
  duplicate     Duplicate a secret
  alias         Alias/symlink a secret
  move          Move a secret
  remove        Remove a secret
  list          List all secrets
  grep          Grep all secrets
  init          Initialize new password store
  clone         Clone existing password store
  sync          Sync password store
  slam          Aggressively lock password store & keys preventing access (emergency)
  totp          Manage TOTP tokens
  recipients    Manage store recipients
  git           Invoke git command in password store
  tomb          Manage password store Tomb
  housekeeping  Housekeeping utilities
  help          Print this message or the help of the given subcommand(s)

Options:
  -f, --force         Force the action, ignore warnings
  -I, --no-interact   Not interactive, do not prompt
  -y, --yes           Assume yes for prompts
  -q, --quiet         Produce output suitable for logging and automation
  -v, --verbose...    Enable verbose information and logging
  -s, --store <PATH>  Password store to use [env: PASSWORD_STORE_DIR=]
      --gpg-tty       Instruct GPG to ask passphrase in TTY rather than pinentry
  -h, --help          Print help
  -V, --version       Print version
```

## License
This project is released under the GNU GPL-3.0 license.
Check out the [LICENSE](LICENSE) file for more information.

The library portion of this project is licensed under the GNU LGPL-3.0 license.
Check out the [lib/LICENSE](lib/LICENSE) file for more information.

[git]: https://git-scm.com/
[github-latest-release]: https://github.com/timvisee/prs/releases/latest
[github-release-latest]: https://github.com/timvisee/prs/releases/latest
[github-releases]: https://github.com/timvisee/prs/releases
[gitlab-packages]: https://gitlab.com/timvisee/prs/-/packages
[gitlab-releases]: https://gitlab.com/timvisee/prs/-/releases
[gpg]: https://gnupg.org/
[homebrew]: https://brew.sh/
[homebrew-install]: https://brew.sh/#install
[linux-notifications]: https://wiki.archlinux.org/index.php/Desktop_notifications
[pass-compatible-clients]: https://www.passwordstore.org#other
[pass]: https://www.passwordstore.org/
[scoop-install]: https://scoop.sh/#installs-in-seconds
[skim]: https://github.com/lotabout/skim
[tomb-faq]: #what-is-tomb
[tomb-install]: https://github.com/dyne/Tomb/blob/master/INSTALL.md
[tomb]: https://www.dyne.org/software/tomb/
[usage-demo-asciinema]: https://asciinema.org/a/368611
[usage-demo-svg]: ./res/demo.svg
[wsl]: https://docs.microsoft.com/en-us/windows/wsl/install-win10
[xkcd538]: https://xkcd.com/538/
