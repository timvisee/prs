# Changelog

## 0.5.4 (2025-09-06)
- Add `--alias` flag to move command, create alias pointing old path to new
- Update MSRV to 1.88
- Update dependencies

## 0.5.3 (2025-08-14)
- Prompt user whether to add new recipient keys to keychain
- Don't delete recipient keys from store if listed in `.gpg-id` file
- Update dependencies

## 0.5.2 (2024-11-09)
- Autocomplete secret names in zsh
- Update dependencies

## 0.5.1 (2024-03-17)
- Add `--pager` alias for `--viewer`
- Add `PRS_PAGER` variable to set custom pager as viewer
- Add scroll support to built-in secure viewer
- Enforce use of TTY when using built-in secure viewer
- Fix errors when `.gpg-id` contains comments
- Fix prs not using existing remote on git repository
- Fix current git branch not using newly configured remote
- Fix auto completion of secrets for bash
- Fix panic on `internal completions` command
- Update dependencies

## 0.5.0 (2023-01-19)
- Add `prs` homebrew package for macOS
- Add `sync status` command to show sync status, changed files and sync command
  hints
- Add `sync commit` command to commit all uncommitted changes in password store
- Add `sync reset` command to reset all uncommitted changes in password store
- Add secure viewer to show secret contents in TTY that clears when closed,
  instead of outputting to stdout
- Add `--viewer` flag to show contents in secure viewer with `show` and other
  commands
- Using `--timeout` now shows contents in secure viewer as this is much more
  reliable
- Add new clipboard handling system making it more secure, reliable and robust
- Make clipboard reverting much more reliable
- Fix clipboard reverting breaking when user copies again before reverting
- Using `totp copy` now recopies the token when it changes within the timeout
- Using `generate -ec` now copies the generated password both before and after
  editing
- Clipboard notifications now show if contents are reverted, replaced or cleared
- Don't fork process on X11 when setting clipboard due to security concerns
- Propagate `--quiet` and `--verbose` flags to clipboard handling and
  notifications
- Fix errors when using relative path for password store with `--store` or
  `PASSWORD_STORE_DIR`
- Fix `slam` errors, don't invoke `gpgconf`, `keychain` or `pkill` binaries if
  they don't exist
- Abort `grep` and `recrypt` commands after 5 failures unless forced
- Fix panic using `show` when secret has non UTF-8 contents
- Fix runtime errors for some regular expressions due to missing Perl features
- Show verbose output and detailed errors when using `--verbose` with
  `gnupg-bin` backend
- Using `git` command now invokes git directly instead of calling through `sh`
  making it more robust
- Make sync remote hint style after `sync init` consistent with other hints
- Don't show `--verbose` and `--force` hints in error output when already used
- Don't allow `--no-interact` together with `--viewer` or `--timeout`
- Rename `prs-cli` to `prs` in help output to be consistent with the binary name
- Remove all `unsafe` usages from codebase
- Show compiler warning when no interaction selection mode feature is used
- Update dependencies

## 0.4.1 (2023-01-11)
- Add `grep` command to search secret contents
- Add `search` alias for `list`
- Add `pwgen` alias for `generate`
- Show progress bar in long tasks such as `housekeeping recrypt` and `grep`
- Show descriptive compiler error when required features aren't selected

## 0.4.0 (2023-01-07)
- Reorder commands in help output to show the most useful commands first
- Add TOTP token support for handling two factor authentication codes
- Add support for Steam TOTP tokens
- Add `totp show` command to show a TOTP token from the store
- Add `totp copy` command to copy a TOTP token to the clipboard
- Add `totp live` command to watch a TOTP token live until cancelled
- Add `totp qr` command to show a TOTP QR code for adding to another authenticator
- Add `slam` command to aggressively close the password store, Tomb, opened GPG
  keys and persistent SSH connections in case of an emergency
- Move `--store` property to root of `prs`, making it globally usable
- For the `generate` command, add `--stdout` alias for `--show`
- Automatically open Tomb when using `sync` command
- Make hints shown with `prs` Tomb aware, preventing weird suggestions when a
  Tomb is closed
- Show warning when using `git` command if sync is not initialised
- Fix Tomb's not closing due to persistent SSH connections, these connections
  are now dropped automatically
- Make interactive selection through skim full screen
- Fix password generator panicking on very short/long lengths
- Improve various error messages making them more descriptive
- Use GnuPG binary backend by default now, rather than GPGME
- Improve GNU and musl CI builds
- Fix errors and warnings for Windows builds
- Remove macOS builds from releases, users can compile from source
- Don't publish release candidate releases on Arch AUR
- Remove unsafe code for handling UTF-16 output
- Remove unsafe code for signalling SSH processes
- Update command-line interface handling system
- Disable unused features in dependencies to shrink dependency count
- Update MSRV to 1.64
- Update dependencies

## 0.3.5 (2022-08-18)
- Add secret autocompletion for bash
- Update MSRV to 1.60
- Fix AUR release
- Update dependencies

## 0.3.4 (2022-06-20)
- Fix Windows release issue
- Update dependencies

## 0.3.3 (2022-06-19)
- Set `GPG_TTY` environment variable in GPGME backend with `--gpg-tty` on supported platforms
- Update Arch AUR packages to latest standards, make tomb an optional dependency
- Resolve dependency vulnerabilities (CVE-2020-36205, CVE-2021-45707)
- Bump MSRV to 1.58.1
- Update dependencies

## 0.3.2 (2021-08-30)
- Fix build error when `tomb` feature is not set

## 0.3.1 (2021-08-30)
- Fix `--gpg-tty` not prompting in tty if `GPG_TTY` was not set
- Update dependencies

## 0.3.0 (2021-08-25)
- Add `--gpg-tty` flag to instruct GPG to ask passphrases in the TTY
- Partially re-enable SSH connection reuse on whitelisted hosts to speed up
  syncing (https://gitlab.com/timvisee/prs/-/issues/31)
- Fix tomb initialisation when not forcing, ask user to force
- Fix permission error when initializing Tomb through temporary store
- Add crypto/GPG config for more internal configurability
- Update dependencies

## 0.2.15 (2021-08-18)
- Add `--merge` to `generate` command, to prevent creating a new secret
- Fix error on secret generation to new file
- Fix `tomb init` error when user has a large password store
- Fix AUR package release
- Update dependencies

## 0.2.14 (2021-07-31)
- Lib: operations on `Plaintext` now borrow instead of move
  (https://github.com/timvisee/prs/pull/9)
- Fix AUR package release
- Bump MSRV to 1.53
- Update dependencies

## 0.2.13 (2021-07-09)
- Fix incorrect Tomb size when resizing because it was closed first
- Fix AUR package release

## 0.2.12 (2021-07-08)
- Add [Tomb](https://www.dyne.org/software/tomb/) support on Linux
  ([info](https://github.com/timvisee/prs#what-is-tomb))
  (https://gitlab.com/timvisee/prs/-/issues/36)
- Add `--copy` flag to show command
- Show error if user tries to generate recipient with `--no-interact`
- Rename secret argument from `DEST` to `NAME` for `add` and `generate` commands
- Do not scan `lost+found` directory for secrets, add to `.gitignore`
- Add compile time feature flag to handle interactive selection with `skim` or
  `fzf` binary
- Update dependencies

## 0.2.11 (2021-04-30)
- Fix panic when generating command completion script to file
  (https://gitlab.com/timvisee/prs/-/issues/35)
- Update dependencies

## 0.2.10 (2021-04-29)
- Add `--no-sync` to prevent syncing and committing when making changes, this
  keeps the store repository dirty
- Add `--allow-dirty` to make changes to the store while the repository is still
  dirty
- Add `print` alias for `show`
- Show full secret name when user query is just a partial match
- Do not show interactive secret selection if no secret matched the user query
- Add `--no-interact` flag to `dmenu` and `rofi` scripts
- Set that `--verbose` flag does not take a value
- Update dependencies

## 0.2.9 (2021-04-23)
- Add `insert` alias for `add`
- Fix panic when generating ZSH shell completions
  (https://github.com/timvisee/prs/issues/7#issuecomment-825482490)
- Improved command-line argument handling, updated `clap` to `v3.0`
- Update dependencies

## 0.2.8 (2021-04-22)
- Add `internal completions` command to generate shell completion scripts
- Output nothing from `list` command if we have no secrets
- Trim end of value when selecting property from secret using `show --property`
- Rename hidden `_internal` command to `internal`
- Update dependencies

## 0.2.7 (2021-03-30)
- Do not allow users to remove last recipient from store, which would
  irrecoverably void it (https://gitlab.com/timvisee/prs/-/issues/32)
- Update dependencies

## 0.2.6 (2021-03-29)
- Fix errors when `git` or `gpg` binary path contains spaces, notably on Windows
- Update dependencies

## 0.2.5 (2021-03-22)
- Disable git SSH connection reuse, until additional logic to handle failures is
  implemented
  (https://github.com/timvisee/prs/issues/5#issuecomment-803940880)
- Update dependencies

## 0.2.4 (2021-03-15)
- Fix error caused by unexpected output from `gpg` binary
- Update dependencies

## 0.2.3 (2021-03-04)
- Show tree style output for `list` command (this changes the default behaviour)
- Add `-l` flag to `list` command to output as plain file list
- Update dependencies

## 0.2.2 (2021-02-28)
- Add `PASSWORD_STORE_DIR` environment variable to customize password store path
- Fix GnuPG binary backend errors on systems providing non-English `LANG`/`LANGUAGE` value to it
- Fix GnuPG binary backend reporting GPGME backend errors
- Improve GnuPG binary output parsing, fall back to UTF-16 decoding
- Update dependencies

## 0.2.1 (2021-02-23)
- Add Wayland support
- Use GnuPG binary backend by default, instead of GPGME backend
- Update dependencies

## 0.2.0 (2021-02-15)
- Add Windows support, and Windows release binary
- Add GPGME cryptography backend for GPG support (default, compiler feature: `backend-gpgme`)
- Add GnuPG binary cryptography backend for GPG support (works on Windows, compiler feature: `backend-gnupg-bin`)
- Add cryptography framework, allow use of different cryptography with different backends
- Fix cancelling interactive secret selection not actually cancelling the action
- Use `fzf` binary instead of `skim` for interactive selection on on non-Unix platforms
- Do not quit with non-zero exit code when no subcommand is given
- Disable colored output on Windows for compatibility
- Always enable `alias` feature by default, instead of doing this just on Unix/Windows
- Update dependencies

## 0.1.7 (2021-02-01)
- Always end secret output to stdout with newline
- Update dependencies

## 0.1.6 (2021-01-18)
- Show or copy a specific secret property with `--property`
- Add optional `--timeout` flag to `show` command, output is cleared afterwards
- Ask to remove pointed to secrets when removing alias secret
- Don't crash on re-encrypt failure, continue and show error summary instead
- Extend list of password generator characters with `[]<>(),.;|`
- Require `--copy` with `--timeout` with `generate` command
- Only run alias management tasks on platforms that support it
- Add `dmenu` and `rofi` scripts to type selected password
- Update dependencies

## 0.1.5 (2021-01-11)
- Generate password instead of passphrase by default with `generate` command
- Add `--length` option to `generate` command
- Do not require to store generated password to secret with `generate` when `--show` or `--copy` is provided
- Generate passphrase with `generate --passphrase`
- Improve secret listing performance
- Update dependencies

## 0.1.4 (2021-01-10)
- Add alias support
  (https://gitlab.com/timvisee/prs/-/issues/9)  
  _You can now easily create aliases for secrets. Aliases are symlinks under the
  hood, compatible with most other `pass` clients. Aliases are automatically
  updated when moving/removing their pointed to secret._
- Add `alias` command to create new aliases
- Add `--aliases` and `--no-aliases` flags to `list` command
- Add `--password` alias for `--first` in `show` command
- Update dependencies

## 0.1.3 (2020-12-14)
- Add dmenu and rofi quick copy scripts
- Use secure directory to edit secret if possible (such as `/dev/shm`)
- Improve clipboard handling on Windows, do not block console when waiting for
  clear timeout.
- Do not try to parse git flags/options passed to `prs git [GIT]` which caused
  errors
- Improve security description in README
- Improve various user prompts
- Fix crash when setting clipboard when it was previously empty
- Fix error on macOS when clearing clipboard after timeout
  (https://gitlab.com/timvisee/prs/-/issues/8)
- Update dependencies

## 0.1.2 (2020-11-09)

- Fix release automation

## 0.1.1 (2020-11-08)

- Update dependencies
- Fix release automation

## 0.1.0 (2020-11-08)

- Initial release
