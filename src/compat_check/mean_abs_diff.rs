use crate::{compat_check::normalize_lists, types};

pub fn calculate_mad(list1: &Vec<types::AnimeScored>, mut list2: Vec<types::AnimeScored>) -> (f64, usize) {
    list2.sort_by_key(|f| f.id);

    let mut list3: Vec<normalize_lists::AnimeScores> = Vec::new();

    let mut index1 = 0;
    let mut index2 = 0;

    while index1 < list1.len() && index2 < list2.len() {
        if list2[index2].id > list1[index1].id {
            index1 += 1;
            continue;
        }
        if list2[index2].id < list1[index1].id {
            index2 += 1;
            continue;
        }
        if list2[index2].id == list1[index1].id {
            let new_entry = normalize_lists::AnimeScores {
                id: list1[index1].id,
                score1: list1[index1].score,
                score2: list2[index2].score,
            };
            list3.push(new_entry);
            index1 += 1;
            index2 += 1;
        }
    }
    let len = list3.len().clone();
    let mut total_difference: f64 = 0.0;
    for entry in list3 {
        let diff = (entry.score1 as i64 - entry.score2 as i64).abs();
        total_difference += diff as f64;
    }
    (total_difference/(len as f64), len)
}