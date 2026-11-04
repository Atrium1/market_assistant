//! Factory recovery credentials — embedded, not editable in UI, not stored in DB.
//!
//! TODO(release): replace before distributing installers.
//! Do not commit real production secrets to a public repository.

pub const FACTORY_USERNAME: &str = "CHANGE_ME_USER";
pub const FACTORY_PASSWORD: &str = "CHANGE_ME_PASS";

pub fn matches_factory(username: &str, password: &str) -> bool {
    constant_time_eq(username.as_bytes(), FACTORY_USERNAME.as_bytes())
        && constant_time_eq(password.as_bytes(), FACTORY_PASSWORD.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
