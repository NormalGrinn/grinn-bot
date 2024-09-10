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
        title {
          romaji
          english
        }
        }
      }
    }
  }
}
";

pub const USERANIMELISTQUERY: &str = "
query ($userName: String)
{
  MediaListCollection (userName: $userName, type:ANIME, status:COMPLETED) {
    lists {
      entries {
        score(format: POINT_100)
        media {
        id
        idMal
        }
      }
    }
    user {
      mediaListOptions {
        scoreFormat
      }
    }
  }
}
";

pub const STAFFQUERY: &str = "
query ($animeID: Int)
{
  Media(id: $animeID) {
    staff {
      edges {
        node {
          name {
            full
          }
        }
        role
      }
    }
  }
}
";

pub const MAINVAQUERY: &str = "
query ($animeID: Int)
{
  Media(id: $animeID) {
    characters(role:MAIN) {
      edges {
        node {
          id
        }
        voiceActors(language:JAPANESE) {
          name {
            full
          }
        }
      }
    }
  }
}
";

pub const STUDIOQUERY: &str = "
query ($animeID: Int)
{
  Media(id: $animeID) {
    studios {
      edges {
        node {
          name
        }
        isMain
      }
    }
  }
}
";