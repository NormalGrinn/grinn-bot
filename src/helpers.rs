use rand::Rng;

use crate::types;

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

pub fn get_typed_hint(v: &mut Vec<types::Hint>, hint_type: &str) -> Option<types::Hint> {
    let mut i = 0;
    while i < v.len() {
        match &v[i] {
            types::Hint::Season(_) => { if hint_type == "Season" { return Some(v.remove(i)) } },
            types::Hint::SeasonYear(_) => { if hint_type == "Year" { return Some(v.remove(i)) } },
            types::Hint::Format(_) => { if hint_type == "Format" { return Some(v.remove(i)) } },
            types::Hint::Genres(_) => { if hint_type == "Genre" { return Some(v.remove(i)) } },
            types::Hint::Studios(_) => { if hint_type == "Studio" { return Some(v.remove(i)) } },
            types::Hint::VoiceActors(_) => { if hint_type == "Voice Actor" { return Some(v.remove(i)) } },
            types::Hint::Tag(_) => { if hint_type == "Tag" { return Some(v.remove(i)) } },
            types::Hint::Staff(_) => { if hint_type == "Staff" { return Some(v.remove(i)) } },
            types::Hint::AverageScore(_) => { if hint_type == "AL Score" { return Some(v.remove(i)) } },
            types::Hint::Source(_) => { if hint_type == "Source" { return Some(v.remove(i)) } },
            types::Hint::UserScore(_) => { if hint_type == "User Score" { return Some(v.remove(i)) } },
        }
        i += 1;
    }
    return None
}