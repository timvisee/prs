# Changelog

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
