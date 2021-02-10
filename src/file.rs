use std::io::Read;

use reqwest::blocking::Client;
use reqwest::header;

use crate::models::HasName;

pub enum File {
    TitleBasics,
    // TitleEpisode,
    // TitleRatings,
}

const BASE_URL: &str = "https://datasets.imdbws.com/";

fn get_download_url<T: HasName>() -> String {
    format!("{}{}", BASE_URL, T::get_name())
}

pub fn get_download_size<T: HasName>() -> Result<usize, reqwest::Error> {
    let client = Client::new();
    let url = get_download_url::<T>();

    let download_size = {
        let resp = client.head(url.as_str()).send()?;

        if resp.status().is_success() {
            resp.headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|ct_len| ct_len.to_str().ok())
                .and_then(|ct_len| ct_len.parse::<usize>().ok())
                .unwrap_or(0)
        } else {
            panic!(format!(
                "Couldn't download URL: {}. Error: {:?}",
                url,
                resp.status(),
            ));
        }
    };

    Ok(download_size)
}

pub fn download<T: HasName>(// _download_size: usize,
   // _progress_callback: impl Fn(usize),
) -> Result<bytes::Bytes, reqwest::Error> {
    let client = Client::new();
    let url = get_download_url::<T>();

    let request = client.get(url.as_str());

    //let mut out = bytes::BytesMut::with_capacity(download_size);
    //let mut buff = [0; 4096];

    let mut download = request.send()?;

    let mut buffer = vec![];
    download.read_to_end(&mut buffer).unwrap();

    Ok(bytes::Bytes::from(buffer))

    /*while let Ok(size) = download.read(&mut buff) { TODO FIX
        if size == 0 {
            break;
        }
        out.extend_from_slice(&buff);
        progress_callback(size);
    }

    Ok(out.freeze())*/
}
