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

pub const USERLISTGUESSINGQUERY: &str = "
query ($userName: String)
{
  MediaListCollection (userName: $userName, type:ANIME, status:COMPLETED) {
    lists {
      entries {
        score(format: POINT_100)
        media {
        id
        season
        seasonYear
        format
        genres
        tags {
          name
          rank
        }
        averageScore
        source
        }
      }
    }
  }
}
";

pub const ANIMESEARCHQUERY: &str = "
query ($animeName: String)
{
  Media (search: $animeName) {
    id
  }
}";