use rand::Rng;

pub fn split_first_word(input: &str) -> (String, String) {
    let mut parts = input.splitn(2, ' ');
    let first_part = parts.next().unwrap_or("").to_string();
    let second_part = parts.next().unwrap_or("").to_string();
    (first_part, second_part)
}

pub fn get_random_element_from_vec<T>(v: &mut Vec<T>) -> Option<T> {
    if v.is_empty() {
        None
    } else {
        let mut rng = rand::thread_rng();
        let chosen_index = rng.gen_range(0..v.len());
        Some(v.remove(chosen_index))
    }
}

pub fn display_str_vec(v: &Vec<String>) -> String {
    let mut result = String::new();
    for hint in v {
        result.push_str(hint);
        result.push('\n');
    }
    result.trim_end().to_string()
}