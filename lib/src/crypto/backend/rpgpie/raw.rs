use std::fmt::Write;

use pgp::{
    Deserializable,
    crypto::{aead::AeadAlgorithm, sym::SymmetricKeyAlgorithm},
};
use rpgpie::policy::Seipd;

use super::context::{Context, Error};
use crate::{Ciphertext, Key, Plaintext, crypto::proto};
use anyhow::Result;

pub fn cards() -> Result<Vec<openpgp_card::Card<openpgp_card::state::Open>>> {
    let mut cards = Vec::new();
    for backend in card_backend_pcsc::PcscBackend::card_backends(None)? {
        let card = openpgp_card::Card::new(backend?)?;
        cards.push(card);
    }

    Ok(cards)
}

pub(super) fn format_fingerprint(fingerprint: pgp::types::Fingerprint) -> String {
    fingerprint
        .as_bytes()
        .iter()
        .fold(String::new(), |mut fp, byte| {
            write!(fp, "{:02x}", byte).unwrap();
            fp
        })
}

pub(super) fn metadata_for_cert(cert: &rpgpie::certificate::Certificate) -> Key {
    let fp = format_fingerprint(cert.fingerprint());
    let ccert = rpgpie::certificate::Checked::new(cert);
    let first = ccert.primary_user_id();
    let user_ids = first
        .into_iter()
        .chain(ccert.user_ids().iter().filter(|u| Some(*u) != first))
        .map(|u| u.id.to_string())
        .collect();

    Key::Gpg(proto::gpg::Key {
        fingerprint: fp,
        user_ids,
    })
}

fn touch_prompt() {
    eprintln!("Please touch the card");
}

fn unlock_pin(card: &mut openpgp_card::Card<openpgp_card::state::Transaction>) -> Result<()> {
    let ident = card.application_identifier()?.ident();
    if let Ok(Some(pin)) = openpgp_card_state::get_pin(&ident) {
        match card.verify_user_pin(pin.into()) {
            Ok(_) => Ok(()),
            Err(err) => {
                openpgp_card_state::drop_pin(&ident)?;
                Err(anyhow::Error::msg(format!(
                    "The stored user pin was rejected by the card '{}'. Dropping the pin from storage. {}",
                    ident, err
                )))
            }
        }
    } else {
        Err(anyhow::Error::msg(format!(
            "No user PIN configured for card '{}'",
            ident,
        )))
    }
}

pub(super) fn decrypt(
    _context: &mut Context,
    data: &[u8],
) -> std::result::Result<Plaintext, Error> {
    let (msg, _header) = pgp::composed::Message::from_reader_single(data)?;

    // TODO: select the correct card
    let cards = card_backend_pcsc::PcscBackend::card_backends(None)
        .map_err(|e| Error::Smartcard(e.into()))?;
    let mut card = if let Some(Ok(card)) = cards.into_iter().next() {
        Ok(openpgp_card::Card::new(card)?)
    } else {
        Err(Error::NoSecretKey)
    }?;

    let mut trans = card.transaction()?;
    unlock_pin(&mut trans)?;

    let slot = openpgp_card_rpgp::CardSlot::init_from_card(
        &mut trans,
        openpgp_card::ocard::KeyType::Decryption,
        &touch_prompt,
    )?;

    let result = slot.decrypt_message(&msg)?;
    let lit = match result.decompress()? {
        pgp::Message::Literal(literal_data) => Ok(literal_data),
        _ => Err(Error::MalformedMessage),
    }?;

    Ok(Plaintext::from(lit.data().to_vec()))
}

pub(super) fn encrypt(
    context: &mut Context,
    fingerprints: &[impl AsRef<str>],
    data: &[u8],
) -> Result<Ciphertext> {
    // TODO: Use a higher abstraction level than this
    let now = chrono::offset::Utc::now();

    let mut symm = rpgpie::policy::PREFERRED_SYMMETRIC_KEY_ALGORITHMS.to_vec();
    let mut aead = rpgpie::policy::PREFERRED_AEAD_ALGORITHMS.to_vec();
    let mut seipd = rpgpie::policy::PREFERRED_SEIPD_MECHANISMS.to_vec();

    let mut keys = Vec::new();
    for fingerprint in fingerprints {
        let cert = context
            .store
            .search_by_fingerprint(fingerprint.as_ref())?
            .first()
            .ok_or(Error::NoSecretKey)?
            .clone();

        // TODO: feature selection should really be done upstream
        let ccert = rpgpie::certificate::Checked::new(&cert);
        if let Some(algo) = ccert.preferred_symmetric_key_algo(&now) {
            symm.retain(|a| algo.contains(a));
        }

        if let Some(algo) = ccert.preferred_aead_algo(&now) {
            aead.retain(|a| algo.contains(a));
        }

        if let Some(p) = ccert.features(&now) {
            fn contains(p: u8, seipd: Seipd) -> bool {
                match seipd {
                    Seipd::SED => true,
                    Seipd::SEIPD1 => p & 1 != 0,
                    Seipd::SEIPD2 => p & 8 != 0,
                }
            }

            seipd.retain(|a| contains(p, *a));
        } else {
            // if there's no features setting, we only do SeipdV1
            seipd = vec![Seipd::SEIPD1]
        }

        let valid_keys = ccert.valid_encryption_capable_component_keys();
        if valid_keys.is_empty() {
            return Err(Error::NoUsablePublicKeys.into());
        } else {
            keys.extend_from_slice(&valid_keys);
        }
    }

    if keys.is_empty() {
        return Err(Error::NoUsablePublicKeys.into());
    }

    let seipd = seipd.first().unwrap_or(&Seipd::SEIPD1);
    let mech = match seipd {
        Seipd::SED => return Err(Error::Unimplemented("SED".to_string()).into()),
        Seipd::SEIPD1 => {
            let algo = symm.first().cloned().unwrap_or_default();
            rpgpie::message::EncryptionMechanism::SeipdV1(algo)
        }
        Seipd::SEIPD2 => {
            let algos = aead
                .first()
                .cloned()
                .unwrap_or((SymmetricKeyAlgorithm::AES128, AeadAlgorithm::Ocb));
            rpgpie::message::EncryptionMechanism::SeipdV2(algos.1, algos.0)
        }
    };

    let mut sink = Vec::new();
    let mut read = std::io::Cursor::new(data);
    let _sek = rpgpie::message::encrypt(
        mech,
        keys,
        Vec::new(),
        Vec::new(),
        None,
        &mut read,
        &mut sink,
        true,
    )?;

    Ok(sink.into())
}
