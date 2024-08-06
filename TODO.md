# Planned features and ideas

- [X] Add '-m/--message' option Implementation idea:
    <https://stackoverflow.com/a/56012454>
- [ ] Add '-n/--dry-run' option
- [ ] Add '-q/--quiet' option
- [X] Use BATS for e2e testing <https://github.com/bats-core/bats-core>
- [ ] Make the binary able to update itself on demand, by pulling from the
    Github release (see <https://github.com/axodotdev/axoupdater>)
- [ ] Configure
  [cargo-dist](https://opensource.axo.dev/cargo-dist/book/introduction.html) for
  automatic builds and installers
- [ ] Configure [oranda](https://opensource.axo.dev/oranda/) for a nice landing
  page
- [ ] Configure [cargo-insta](https://insta.rs/docs/) for e2e tests
- [ ] Fix the CI build of `aarch64` platforms (see <https://github.com/axodotdev/cargo-dist/issues/74>)
