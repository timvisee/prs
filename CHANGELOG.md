# Changelog

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
