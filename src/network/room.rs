use rand::Rng;

/// Generate a random 4-character room code
pub fn generate_room_code() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    (0..4)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

/// Validate a room code format
#[allow(dead_code)]
pub fn is_valid_room_code(code: &str) -> bool {
    code.len() == 4 && code.chars().all(|c| c.is_ascii_alphanumeric())
}
