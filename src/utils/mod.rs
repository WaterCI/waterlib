use rand::Rng;

pub mod docker;

const UUID_PARTS_LEN: u8 = 4;
const UUID_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

fn gen_random_chars(len: usize, charset: &[u8]) -> String {
    let mut rng = rand::thread_rng();

    (0..len)
        .map(|_| charset[rng.gen_range(0..charset.len())] as char)
        .collect::<String>()
}

pub fn gen_uuid() -> String {
    format!(
        "{}-{}-{}-{}",
        gen_random_chars(UUID_PARTS_LEN as usize, UUID_CHARSET),
        gen_random_chars(UUID_PARTS_LEN as usize, UUID_CHARSET),
        gen_random_chars(UUID_PARTS_LEN as usize, UUID_CHARSET),
        gen_random_chars(UUID_PARTS_LEN as usize, UUID_CHARSET),
    )
}
