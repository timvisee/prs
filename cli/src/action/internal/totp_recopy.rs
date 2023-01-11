use std::io;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use base64::Engine;
use clap::ArgMatches;
use prs_lib::Plaintext;
use thiserror::Error;

use crate::cmd::matcher::{internal::totp_recopy::TotpRecopyMatcher, MainMatcher, Matcher};
use crate::util::{clipboard, totp::Totp};

/// A internal TOTP recopy action.
pub struct TotpRecopy<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> TotpRecopy<'a> {
    /// Construct a new clipboard revert action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the clipboard revert action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_totp_recopy = TotpRecopyMatcher::with(self.cmd_matches).unwrap();

        // Grab clipboard data from stdin
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        let totp = base64::engine::general_purpose::STANDARD
            .decode(buffer.trim())
            .map_err(|err| Err::Data(anyhow!(err)))?;
        let totp = std::str::from_utf8(&totp).map_err(|err| Err::Data(anyhow!(err)))?;
        let totp = Totp::from_url(totp).map_err(Err::Data)?;
        drop(Plaintext::from(buffer));

        // Determine until time based on timeout
        let timeout = matcher_totp_recopy.timeout().unwrap();
        let until = Instant::now() + Duration::from_secs(timeout);

        // Keep recopying chaning token until the copy timeout is reached
        while until > Instant::now() {
            // Calculate remaining timeout time, get current TOTP TTL
            let remaining_timeout = until.duration_since(std::time::Instant::now());
            let token = totp.generate_current().map_err(Err::Totp)?;
            let ttl = totp.ttl().map_err(Err::Totp)?;

            // Keep clipboard timeout within timeout remaining and current toeken TTL if recopying
            clipboard::copy_plaintext(
                token.clone(),
                false,
                !matcher_main.force(),
                matcher_main.quiet(),
                matcher_main.verbose(),
                remaining_timeout.as_secs() + 1,
            )?;

            // Wait for timeout, stop if clipboard was changed
            let ttl_duration = Duration::from_secs(ttl);
            if clipboard::timeout_or_clip_change(&token, ttl_duration) {
                if matcher_main.verbose() {
                    eprintln!("Clipboard changed, TOTP copy stopped");
                }
                break;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to obtain TOTP from stdin, malformed data")]
    Data(#[source] anyhow::Error),

    #[error("failed to generate TOTP token")]
    Totp(#[source] anyhow::Error),
}
