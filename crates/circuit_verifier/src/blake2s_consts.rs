pub use circuits::blake::BLAKE2S_IV;

pub fn blake2s_initial_state() -> [u32; 8] {
    let mut h = BLAKE2S_IV;
    h[0] ^= 0x01010020;
    h
}
