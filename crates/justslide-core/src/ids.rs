use rand::Rng;

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

/// Opaque, persistent identifiers matching the registry patterns
/// (`^ws_[a-z0-9]{8,32}$`, `^tab_...`, `^fav_...`).
pub fn new_id(prefix: &str) -> String {
    let mut rng = rand::rng();
    let body: String = (0..16).map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char).collect();
    format!("{prefix}_{body}")
}

/// 256-bit URL-safe random secret (43 base64url chars) for pairing tokens.
pub fn new_secret() -> String {
    use base64::Engine;
    let mut b = [0u8; 32];
    rand::rng().fill(&mut b);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b)
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// RFC 4122 version-4 UUID string (used for WebKit data-store identifiers).
pub fn uuid_v4() -> String {
    let mut b = [0u8; 16];
    rand::rng().fill(&mut b);
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32])
}
