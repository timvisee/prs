use crate::cmd::matcher::MainMatcher;
use prs_lib::crypto::{self, Config, Context, Proto};

/// Default cryptography protocol.
const PROTO: Proto = Proto::Gpg;

/// Construct crypto config, respect CLI arguments.
pub(crate) fn config(matcher_main: &MainMatcher) -> Config {
    let mut config = Config::from(PROTO);
    config.gpg_tty = matcher_main.gpg_tty();
    config
}

/// Construct crypto context, respect CLI arguments.
pub(crate) fn context(matcher_main: &MainMatcher) -> Result<Context, crypto::Err> {
    let config = config(matcher_main);
    crypto::context(&config)
}
