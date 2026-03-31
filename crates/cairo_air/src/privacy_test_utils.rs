use circuits::blake::HashValue;
use stwo::core::fields::qm31::QM31;

pub fn hash_to_u32s(hash: HashValue<QM31>) -> [u32; 8] {
    [
        hash.0.0.0.0,
        hash.0.0.1.0,
        hash.0.1.0.0,
        hash.0.1.1.0,
        hash.1.0.0.0,
        hash.1.0.1.0,
        hash.1.1.0.0,
        hash.1.1.1.0,
    ]
}

pub fn update_const_in_privacy_rs(const_name: &str, value: [u32; 8]) {
    let privacy_rs = format!("{}/src/privacy.rs", env!("CARGO_MANIFEST_DIR"));
    let content = std::fs::read_to_string(&privacy_rs).unwrap();
    let marker = format!("pub const {const_name}: [u32; 8] =");
    let start = content.find(&marker).unwrap();
    let eq_pos = content[start..].find('=').unwrap() + start;
    let array_start = content[eq_pos..].find('[').unwrap() + eq_pos;
    let array_end = content[array_start..].find(']').unwrap() + array_start + 1;
    let new_array = format!(
        "[{}, {}, {}, {}, {}, {}, {}, {}]",
        value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7]
    );
    let new_content = format!("{}{}{}", &content[..array_start], new_array, &content[array_end..]);
    std::fs::write(&privacy_rs, new_content).unwrap();
}
