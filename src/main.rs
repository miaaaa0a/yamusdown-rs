use std::{env, error::Error};

use clap::Parser;

mod api;
mod cli_args;
mod download;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // parse cli arguments
    let args = cli_args::Args::try_parse()?;
    //println!("{}", api::get_id(args.url));
    let (kind, id) = api::get_kind(args.url);

    // get yandex music token
    dotenvy::dotenv()?;
    let token = env::var("TOKEN").expect("eated the token :(");

    /*let body = api::download_info(&id, token).await?;
    let track = api::download_track(body).await?;

    std::fs::write(
        format!("meow.{}", track.1.file_format()),
        track.0
    )?;*/
    download::download_media(id, kind, token).await?;

    Ok(())
}
