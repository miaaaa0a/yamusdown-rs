use aes::Aes128;
use anyhow::{Result, anyhow};
use base64::prelude::*;
use ctr::cipher::{KeyIvInit, StreamCipher};
use hex::FromHex;
use hmac::{Hmac, Mac};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde_json::Value;
use sha2::Sha256;
use std::time::SystemTime;

type Aes128Ctr = ctr::Ctr128BE<Aes128>;
type HMac256 = Hmac<Sha256>;

enum Container {
    FLAC,
    MP3,
    M4A
}

enum Codec {
    FLAC,
    MP3,
    AAC
}

pub struct Format(Container, Codec);
impl Format {
    pub fn file_format(&self) -> String {
        match self.0 {
            Container::FLAC => "flac",
            Container::MP3 => "mp3",
            Container::M4A => "m4a"
        }.to_string()
    }
}

fn codec2format(codec: &str) -> Option<Format> {
    match codec {
        "flac" => Some(Format(Container::FLAC, Codec::FLAC)),
        "flac-mp4" => Some(Format(Container::M4A, Codec::FLAC)),
        "mp3" => Some(Format(Container::MP3, Codec::MP3)),
        "aac" => Some(Format(Container::M4A, Codec::AAC)),
        "he-aac" => Some(Format(Container::M4A, Codec::AAC)),
        "aac-mp4" => Some(Format(Container::M4A, Codec::AAC)),
        "he-aac-mp4" => Some(Format(Container::M4A, Codec::AAC)),
        &_ => None
    }
}

pub async fn download_info(track_id: &str, token: String) -> Result<Value> {
    let client = reqwest::Client::new();

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs()
        .to_string();
    let quality = "lossless";
    let codecs = "flac,flac-mp4,mp3,aac,he-aac,aac-mp4,he-aac-mp4";
    let transports = "encraw";

    let hmac_input = format!("{timestamp}{track_id}{quality}{}{transports}", codecs.replace(",", ""));

    let mut mac = HMac256::new_from_slice(b"p93jhgh689SBReK6ghtw62")?;
    mac.update(hmac_input.as_bytes());

    let mut sign = BASE64_STANDARD.encode(mac.finalize().into_bytes());
    sign.pop(); // Match Python behavior

    let params: Vec<(&str, &str)> = vec![
        ("ts", &timestamp),
        ("trackId", track_id),
        ("quality", quality),
        ("codecs", codecs),
        ("transports", transports),
        ("sign", &sign),
    ];

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("OAuth {token}"))?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("Yandex-Music-API"));
    headers.insert(
        "X-Yandex-Music-Client",
        HeaderValue::from_static("YandexMusicAndroid/24023621"),
    );

    let url =
        reqwest::Url::parse_with_params("https://api.music.yandex.net/get-file-info", &params)?;
    let res = client.get(url).headers(headers).send().await?;

    let body = &res.json::<Value>().await?["result"]["downloadInfo"];
    println!("Response body: {body}");

    Ok(body.clone())
}

fn decrypt_data(data: bytes::Bytes, key: String) -> Result<Vec<u8>> {
    let key_bytes = <[u8; 16]>::from_hex(&key)?;
    let iv = [0u8; 16];
    let mut cipher = Aes128Ctr::new(&key_bytes.into(), &iv.into());
    let mut buf = data.to_vec();
    cipher.apply_keystream(&mut buf);
    Ok(buf)
}

pub async fn download_track(download_info: Value) -> Result<(Vec<u8>, Format)> {
    let data = reqwest::get(download_info["urls"][0].as_str().unwrap().to_string())
        .await?
        .bytes()
        .await?;
    let decrypted = decrypt_data(data, download_info["key"].as_str().unwrap().to_string())
        .map_err(|e| anyhow!(e))?;
    let file_format = codec2format(download_info["codec"].as_str().unwrap()).unwrap();
    Ok((decrypted, file_format))
}
