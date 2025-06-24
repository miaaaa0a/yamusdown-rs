use std::{env, error::Error};

mod api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    let token = env::var("TOKEN").expect("eated the token :(");

    let body = api::download_info("64922264", token).await?;
    let track = api::download_track(body).await?;

    std::fs::write("meow.mp3", track)?;

    Ok(())
}
