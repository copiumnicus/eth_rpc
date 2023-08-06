/// convert hex str to a vec of bytes
pub fn to_vec(mut s: &str) -> Result<Vec<u8>, String> {
    if s.starts_with("0x") {
        s = &s[2..]
    }
    if s.len() % 2 != 0 {
        return Err(format!(
            "Given hex str, is not zero mod 2! len: {:?}",
            s.len()
        ));
    }
    Ok((0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect())
}
