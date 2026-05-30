// Crypto glue for lestrrat-go/jwx exact implementation of JWE and JWS
// ALG: PBES2-HS512+A256KW
// See github.com/lestrrat-go/jwx/v2/jwe

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, Payload},
};
use aes_kw::KwAes256;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as B64};
use hmac::{Hmac, KeyInit as _};
use sha2::Sha512;

const ALG: &str = "PBES2-HS512+A256KW";
const ENC: &str = "A256GCM";
const P2C: u32 = 10_000;

#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    #[error("crypto rng error: {0}")]
    Rng(#[from] getrandom::Error),
    #[error("key wrap error: {0}")]
    KeyWrap(#[from] aes_kw::Error),
    #[error("aead error: {0}")]
    Aead(String),
    #[error("format error: {0}")]
    Format(String),
}

// ref: defradb/keyring/file.go -> in Set
pub fn encrypt(plain_payload: &[u8], password: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut p2s = [0u8; 16];
    let mut iv = [0u8; 12];
    getrandom::fill(&mut p2s)?;
    getrandom::fill(&mut iv)?;

    let header_json = serde_json::json!({
        "alg": ALG,
        "enc": ENC,
        "p2c": P2C,
        "p2s": B64.encode(p2s),
    })
    .to_string();
    let header_b64 = B64.encode(header_json.as_bytes());

    let mut salt = Vec::with_capacity(ALG.len() + 1 + p2s.len());
    salt.extend_from_slice(ALG.as_bytes());
    salt.push(0x00);
    salt.extend_from_slice(&p2s);

    let mut wkey = [0u8; 32];
    pbkdf2::pbkdf2::<Hmac<Sha512>>(password, &salt, P2C, &mut wkey)
        .map_err(|err| CryptoError::Format(err.to_string()))?;

    let mut cek = [0u8; 32];
    getrandom::fill(&mut cek)?;

    let kek = KwAes256::new(&wkey.into());
    let mut encrypted_key = [0u8; 40];
    kek.wrap_key(&cek, &mut encrypted_key)?;

    let cipher = Aes256Gcm::new(&cek.into());
    let mut sealed = cipher
        .encrypt(
            Nonce::from_slice(&iv),
            Payload {
                msg: plain_payload,
                aad: header_b64.as_bytes(),
            },
        )
        .map_err(|err| CryptoError::Aead(err.to_string()))?;

    let tag = sealed.split_off(sealed.len() - 16);
    let cipher_text = sealed;

    let jwe = format!(
        "{}.{}.{}.{}.{}",
        header_b64,
        B64.encode(encrypted_key),
        B64.encode(iv),
        B64.encode(&cipher_text),
        B64.encode(&tag),
    ).as_bytes().to_vec();

    Ok(jwe)
}

// ref: defradb/keyring/file.go -> in Get
pub fn decrypt(compat: &[u8], password: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let s = std::str::from_utf8(compat).map_err(|err| CryptoError::Format(err.to_string()))?;

    let mut parts = s.split(".");
    let header_b64 = parts
        .next()
        .ok_or(CryptoError::Format("missing header".to_string()))?;
    let enc_key = B64
        .decode(
            parts
                .next()
                .ok_or(CryptoError::Format("missing encrypted key".to_string()))?,
        )
        .map_err(|err| CryptoError::Format(err.to_string()))?;
    let iv = B64
        .decode(
            parts
                .next()
                .ok_or(CryptoError::Format("missing iv".to_string()))?,
        )
        .map_err(|err| CryptoError::Format(err.to_string()))?;
    let cipher_text = B64
        .decode(
            parts
                .next()
                .ok_or(CryptoError::Format("missing cipher text".to_string()))?,
        )
        .map_err(|err| CryptoError::Format(err.to_string()))?;
    let tag = B64
        .decode(
            parts
                .next()
                .ok_or(CryptoError::Format("missing tag".to_string()))?,
        )
        .map_err(|err| CryptoError::Format(err.to_string()))?;

    let header_json = B64
        .decode(header_b64)
        .map_err(|err| CryptoError::Format(err.to_string()))?;
    let hdr: serde_json::Value =
        serde_json::from_slice(&header_json).map_err(|err| CryptoError::Format(err.to_string()))?;

    let p2c = hdr["p2c"]
        .as_u64()
        .ok_or(CryptoError::Format("missing p2c".to_string()))? as u32;
    if p2c > 10_000_000 {
        return Err(CryptoError::Format(
            "p2c too large (https://nvd.nist.gov/vuln/detail/CVE-2023-49290)".to_string(),
        ));
    }

    let p2s = B64
        .decode(
            hdr["p2s"]
                .as_str()
                .ok_or(CryptoError::Format("missing p2s".to_string()))?,
        )
        .map_err(|err| CryptoError::Format(err.to_string()))?;

    let mut salt = Vec::with_capacity(ALG.len() + 1 + p2s.len());
    salt.extend_from_slice(ALG.as_bytes());
    salt.push(0x00);
    salt.extend_from_slice(&p2s);

    let mut wkey = [0u8; 32];
    pbkdf2::pbkdf2::<Hmac<Sha512>>(password, &salt, p2c, &mut wkey)
        .map_err(|err| CryptoError::Format(err.to_string()))?;

    let kek = KwAes256::new(&wkey.into());
    let mut cek = [0u8; 32];
    kek.unwrap_key(&enc_key, &mut cek)?;

    let mut buf = cipher_text;
    buf.extend_from_slice(&tag);
    let cipher = Aes256Gcm::new(&cek.into());

    cipher
        .decrypt(
            Nonce::from_slice(&iv),
            Payload {
                msg: &buf,
                aad: header_b64.as_bytes(),
            },
        )
        .map_err(|err| CryptoError::Aead(err.to_string()))
}
