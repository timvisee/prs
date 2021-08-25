# Connection reuse

`prs` automatically tries to reuse SSH connections to greatly speed up
operations in a password store that has sync enabled.

Support for this is limited depending on the used operating system and remote
host. A list of [requirements](#requirements) has been set up for this.
Based on this, `prs` guesses whether connection reuse is supported and
automatically enables it when that is the case.

This feature uses the `ControlPersist` feature within OpenSSH. An open connection
is stored in a file at `/tmp/.prs-session--*`. OpenSSH manages the connection
and the file.

If you're experiencing problems with this feature, please
[disable](#how-to-disable) it and open an issue.

This implementation is still limited and may be troublesome, it requires
additional work. A better way to configure and determine whether connection
reuse is supported should be implemented. Please see the following issues:

- https://gitlab.com/timvisee/prs/-/issues/31
- https://github.com/timvisee/prs/issues/5

## Requirements

You must meet these requirements for connection reuse to be used:

- Password store must have sync enabled (with `git`)
- Password store must use SSH remote
- Only supported on Unix platforms
- Environment variable `GIT_SSH_COMMAND` must not be set
- All password store git remotes that use SSH must have
  [whitelisted](#host-whitelist) domain

## Host whitelist

The following hosts are whitelisted:

```
github.com
gitlab.com
```

_To add a host to this whitelist, please contribute to this project or open an
issue. See `SSH_PERSIST_HOST_WHITELIST` in
[`lib/src/util/git.rs`](../lib/src/util/git.rs)._

## How to disable

There's no need to manually disable this if you don't meet the
[requirements](#requirements).

To disable this feature, you may set the `GIT_SSH_COMMAND` variable:

```bash
# Use default ssh connection in git, disables prs connection reuse
export GIT_SSH_COMMAND=ssh
```
