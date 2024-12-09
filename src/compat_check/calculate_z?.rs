// use crate::types;

// #[derive(Debug)]
// struct AnimeScores {
//     id: u64,
//     score1: u64,
//     score2: u64,
// }

// pub fn calculate_z(mut list1: Vec<types::AnimeScored>, mut list2: Vec<types::AnimeScored>) -> f64 {
//     list1.sort_by_key(|f| f.id);
//     list2.sort_by_key(|f| f.id);

//     let mut list3: Vec<AnimeScores> = Vec::new();
    
//     let mut index1 = 0;
//     let mut index2 = 0;

//     while index1 < list1.len() && index2 < list2.len() {
//         if list2[index2].id > list1[index1].id {
//             index1 += 1;
//             continue;
//         }
//         if list2[index2].id < list1[index1].id {
//             index2 += 1;
//             continue;
//         }
//         if list2[index2].id == list1[index1].id {
//             let new_entry = AnimeScores {
//                 id: list1[index1].id,
//                 score1: list1[index1].score,
//                 score2: list2[index2].score,
//             };
//             list3.push(new_entry);
//             index1 += 1;
//             index2 += 1;
//         }
//     }

//     let mut average1: f64 = 0.0;
//     let mut average2: f64 = 0.0;

//     for i in 0..list1.len() {
//         average1 += list1[i].score as f64;
//     }
//     for i in 0..list2.len() {
//         average2 += list2[i].score as f64;
//     }

//     average1 = average1/(list1.len() as f64);
//     average2 = average2/(list2.len() as f64);

//     let mut standev1: f64 = 0.0;
//     let mut standev2: f64 = 0.0;

//     for i in 0..list1.len() {
//         standev1 += (list1[i].score as f64 - average1).powf(2.0);
//     }
//     for i in 0..list2.len() {
//         standev2 += (list2[i].score as f64 - average2).powf(2.0);
//     }

//     standev1 = (standev1/(list3.len() as f64 - 1.0)).sqrt();
//     standev2 = (standev2/(list3.len() as f64 - 1.0)).sqrt();

//     let mut diff: f64 = 0.0;

//     for i in 0..list3.len() {
//         let x = (list3[i].score1 as f64 - average1) / standev1;
//         let y = (list3[i].score2 as f64 - average2) / standev2;
//         let z = (x-y).abs();
//         diff += z;
//     }
//     diff = diff / list3.len() as f64;
//     diff
// }