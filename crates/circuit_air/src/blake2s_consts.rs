pub const BLAKE2S_IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

pub fn blake2s_initial_state() -> [u32; 8] {
    let mut h = BLAKE2S_IV;
    h[0] ^= 0x01010020;
    h
}
