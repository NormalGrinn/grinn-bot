use crate::types;

#[derive(Debug)]
pub struct AnimeScores {
    pub id: u64,
    pub score1: u64,
    pub score2: u64,
}

#[derive(Debug)]
pub struct NormalizedScores {
    pub score1: f64,
    pub score2: f64,
}

pub fn normalize(list1: &Vec<types::AnimeScored>, list2: &Vec<types::AnimeScored>) -> Vec<NormalizedScores> {
    let mut list3: Vec<AnimeScores> = Vec::new();
    
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
            let new_entry = AnimeScores {
                id: list1[index1].id,
                score1: list1[index1].score,
                score2: list2[index2].score,
            };
            list3.push(new_entry);
            index1 += 1;
            index2 += 1;
        }
    }

    let mut average1: f64 = 0.0;
    let mut average2: f64 = 0.0;
    
    for i in 0..list3.len() {
        average1 += list3[i].score1 as f64;
        average2 += list3[i].score2 as f64;
    }

    average1 = average1/(list3.len() as f64);
    average2 = average2/(list3.len() as f64);

    let mut standev1: f64 = 0.0;
    let mut standev2: f64 = 0.0;

    for i in 0..list3.len() {
        standev1 += (list3[i].score1 as f64 - average1).powf(2.0);
        standev2 += (list3[i].score2 as f64 - average2).powf(2.0);
    }

    standev1 = (standev1/(list3.len() as f64 - 1.0)).sqrt();
    standev2 = (standev2/(list3.len() as f64 - 1.0)).sqrt();

    let mut res_list: Vec<NormalizedScores> = Vec::new();
    for i in 0..list3.len() {
        let x = (list3[i].score1 as f64 - average1) / standev1;
        let y = (list3[i].score2 as f64 - average2) / standev2;
        let new_entry = NormalizedScores {
            score1: x,
            score2: y,
        };
        res_list.push(new_entry);
    }
    res_list
}