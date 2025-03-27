use crate::types;

/*
    Expects a sorted list of entries
*/
pub fn create_ranking_field(entries: &[types::ListEntry]) -> String {
    let mut field: String = "```\n".to_string();
    let header = format!("{:<21} {:5} {:4} {:1}\n", "Username", "Score", "Stat", "Fav");
    field.push_str(&header);
    for e in entries {
        let disp_fav: String;
        if e.is_favourite {disp_fav = "⭐".to_string()} else {disp_fav = "".to_string()};
        let row = format!("{:<21} {:5} {:4} {:1}\n", 
        e.user_name, e.display_score(), e.completion_status.short_display(), disp_fav);
        field.push_str(&row);
    }
    field.push_str("```");
    field
}