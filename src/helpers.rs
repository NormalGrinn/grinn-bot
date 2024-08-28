pub fn split_first_word(input: &str) -> (String, String) {
    let mut parts = input.splitn(2, ' ');
    let first_part = parts.next().unwrap_or("").to_string();
    let second_part = parts.next().unwrap_or("").to_string();
    (first_part, second_part)
}