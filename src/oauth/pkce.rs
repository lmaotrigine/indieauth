use base64::engine::{general_purpose::URL_SAFE_NO_PAD as BASE64_URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use ring::digest::{digest, SHA256};

const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
    abcdefghijklmnopqrstuvwxyz\
    0123456789\
    -.~_";

fn code_verifier(length: usize) -> Vec<u8> {
    assert!(
        (43..=128).contains(&length),
        "code verifier length must be between 43 and 128 characters"
    );
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())])
        .collect()
}

fn b64_urlencode(input: &[u8]) -> String {
    BASE64_URL_SAFE_NO_PAD
        .encode(input)
        .chars()
        .filter_map(|c| match c {
            '=' => None,
            '+' => Some('-'),
            '/' => Some('_'),
            c => Some(c),
        })
        .collect()
}

fn code_challenge(code_verifier: &[u8]) -> String {
    b64_urlencode(digest(&SHA256, code_verifier).as_ref())
}

/// Generate a (code_verifier, code_challenge) pair.
pub fn generate() -> (String, String) {
    let code_verifier = code_verifier(128);
    let code_challenge = code_challenge(&code_verifier);
    (String::from_utf8(code_verifier).unwrap(), code_challenge)
}

/// Verify a code_verifier against a code_challenge.
pub fn verify(verifier: &str, challenge: &str) -> bool {
    code_challenge(verifier.as_bytes()) == challenge
}
