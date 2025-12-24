pub fn prepend_zx(s: &str) -> String {
    if s.len() > 2 && !s.starts_with("0x") {
        format!("0x{}", s)
    } else {
        s.to_string()
    }
}

