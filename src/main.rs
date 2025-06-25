use std::{env, error::Error};

mod api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    let token = env::var("TOKEN").expect("eated the token :(");

    let body = api::download_info("69514380", token).await?;
    let track = api::download_track(body).await?;

    std::fs::write(
        format!("meow.{}", track.1.file_format()),
        track.0
    )?;

    Ok(())
}
