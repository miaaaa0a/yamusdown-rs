use serde_json::Value;
use std::error::Error;

use crate::api::{self, MediaType};

struct TrackInfo {
    pub artist: String,
    pub title: String,
    pub album: String,
    pub track_position: String,
    pub year: String,
    pub cover_url: String,
}
impl TrackInfo {
    pub fn new() -> Self {
        Self {
            artist: String::new(),
            title: String::new(),
            album: String::new(),
            track_position: String::new(),
            year: String::new(),
            cover_url: String::new(),
        }
    }
    pub fn parse_result(&mut self, result: &Value) {
        // omitting multiple artists bad
        let artists = result["artists"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["name"].to_string().trim_matches('"').to_string())
            .collect::<Vec<String>>()
            .join("; ")
            .to_string();
        self.artist = artists;
        self.title = result["title"].to_string().trim_matches('"').to_string();
        // not sure in what case a track would belong in multiple albums?
        self.album = result["albums"][0]["title"]
            .to_string()
            .trim_matches('"')
            .to_string();
        self.track_position = result["albums"][0]["trackPosition"]["index"].to_string();
        self.year = result["albums"][0]["year"].to_string();
        self.cover_url = format!(
            "https://{}",
            result["coverUri"].to_string().trim_matches('"')
        );
    }
}

pub async fn download_media(
    id: String,
    kind: MediaType,
    token: String,
) -> Result<(), Box<dyn Error>> {
    match kind {
        MediaType::Track => {
            let body = api::download_info(&id, &token).await?;
            let track = api::download_track(body).await?;
            // since theres one track we can just grab the first item
            let unparsed_info = &api::tracks_info(vec![id], &token).await?[0];
            let mut track_info = TrackInfo::new();
            track_info.parse_result(unparsed_info);
            let file_name = format!(
                "{:0>2}. {} - {}.{}",
                track_info.track_position,
                track_info.artist,
                track_info.title,
                track.1.file_format()
            );

            std::fs::write(file_name, track.0)?;
        }
        MediaType::Album => {
            println!("type: album");
            let tracks = api::album_tracks(id, &token).await?;
            for track in tracks.as_array().unwrap().iter() {
                println!("downloading track");
                // this is the result of my stupid ahh manager Dr. Borrow Checker
                let mut track_id = track["id"].to_string();
                track_id = track_id.trim_matches('"').to_string();

                let info = api::download_info(&track_id, &token).await?;
                let data = api::download_track(info).await?;

                let unparsed_info = &api::tracks_info(vec![track_id], &token).await?[0];
                let mut track_info = TrackInfo::new();
                track_info.parse_result(unparsed_info);

                // for now we are saving to the same directory
                // this'll be fixed when we set up proper download folder configuration
                let file_name = format!(
                    "{:0>2}. {} - {}.{}",
                    track_info.track_position,
                    track_info.artist,
                    track_info.title,
                    data.1.file_format()
                );
                println!("{}", file_name);
                std::fs::write(file_name, data.0)?;
            }
        }
        MediaType::Artist => todo!(),
        MediaType::Playlist => todo!(),
    };
    Ok(())
}
