use crate::types;

use super::normalize_lists;

#[derive(Debug)]
struct AnimeScores {
    id: u64,
    score1: u64,
    score2: u64,
}

pub fn calculate_cosine_sim(list1: &Vec<types::AnimeScored>, mut list2: Vec<types::AnimeScored>) -> (f64, usize) {
    list2.sort_by_key(|f| f.id);

    let list3: Vec<normalize_lists::NormalizedScores> = normalize_lists::normalize(list1, &list2);


    let mut a: Vec<f64> = Vec::new();
    let mut b: Vec<f64> = Vec::new();

    for i in 0..list3.len() {
        a.push(list3[i].score1);
        b.push(list3[i].score2);
    }

    let mut dot_product = 0.0;
    let mut magnitude_a = 0.0;
    let mut magnitude_b = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        magnitude_a += a[i].powi(2);
        magnitude_b += b[i].powi(2);
    }
    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return (0.0, list3.len()); // Avoid division by zero
    }
    (dot_product / (magnitude_a.sqrt() * magnitude_b.sqrt()), list3.len())
}