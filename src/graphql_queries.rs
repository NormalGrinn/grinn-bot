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
          native
        }
        synonyms
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
query ($animeId: Int)
{
  Media(id: $animeId, type: ANIME) {
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
query ($animeId: Int)
{
  Media(id: $animeId, type: ANIME) {
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
query ($animeId: Int)
{
  Media(id: $animeId, type: ANIME) {
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