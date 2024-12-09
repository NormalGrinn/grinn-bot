use crate::{compat_check::normalize_lists, types};

pub fn calculate_mad_norm(list1: &Vec<types::AnimeScored>, mut list2: Vec<types::AnimeScored>) -> (f64, usize) {
    list2.sort_by_key(|f| f.id);

    let list3: Vec<normalize_lists::NormalizedScores> = normalize_lists::normalize(list1, &list2);
    let len = list3.len();
    let mut total_difference: f64 = 0.0;
    for entry in list3 {
        total_difference += (entry.score1 - entry.score2).abs();
    }
    (total_difference/len as f64, len)
}