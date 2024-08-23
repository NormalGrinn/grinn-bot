pub const YEARQUERY: &str = "
query ($page: Int, $seasonYear: Int)
  {
    Page (page: $page, perPage:50) 
    {
      media (seasonYear: $seasonYear){
        title {
          romaji
        }
        episodes
        duration
      }
    }
  }
";