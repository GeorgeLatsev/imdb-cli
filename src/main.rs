#[macro_use]
extern crate clap;

use std::{sync::Arc, thread};

use clap::{App, Arg, SubCommand};
use indicatif::ProgressBar;
use models::{Episode, HasName, Rating, Title};

mod file;
mod hash;
mod models;
mod store;
mod update;

fn main() {
    let matches = App::new("IMDB CLI")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Easy-to-use store for your movies and series")
        // import export?
        .subcommand(
            SubCommand::with_name("search").arg(Arg::with_name("keywords").required(true).index(1)),
        )
        .subcommand(SubCommand::with_name("info").arg(Arg::with_name("id").required(true).index(1)))
        .subcommand(
            SubCommand::with_name("watched")
                .subcommand(
                    SubCommand::with_name("add").arg(Arg::with_name("id").required(true).index(1)), // more?
                )
                .subcommand(
                    SubCommand::with_name("remove")
                        .arg(Arg::with_name("id").required(true).index(1)),
                )
                .subcommand(
                    SubCommand::with_name("list").arg(
                        Arg::with_name("type")
                            .required(false)
                            .possible_values(&["movie", "series"]),
                    ),
                ),
        )
        .subcommand(SubCommand::with_name("update"))
        .get_matches();

    match matches.subcommand() {
        ("update", _) => {
            let download_size = file::get_download_size::<Title>().unwrap();

            let pb = Arc::new(ProgressBar::new(download_size as u64));

            let c_pb = pb.clone();
            let handle = thread::spawn(move || {
                let bytes = file::download::<Title>(/*download_size, |new| {
                    c_pb.inc(new as u64);
                }*/)
                .unwrap();

                // let hash = hash::calculate_hash(bytes.clone());

                // let last_hash = store::get_last_update_hash(file::File::TitleBasics).unwrap();

                store::update::<Title>(bytes).unwrap();

                c_pb.finish();
            });

            handle.join().unwrap();

            let db = sled::open("imdb-db").unwrap();
            let file_tree = db.open_tree(Title::get_name().as_bytes()).unwrap();

            println!("{:?}", file_tree.len());

            // thread::(move || pb.join());
            // update_handler();
        }
        ("info", Some(matches)) => {
            let id = matches.value_of("id").unwrap();

            info_handler(id);
        }
        ("watched", Some(watched_matches)) => {
            match watched_matches.subcommand() {
                ("list", _) => {
                    watched_list_handler();
                }
                ("add", Some(matches)) => {
                    let id = matches.value_of("id").unwrap();

                    let db = sled::open("my_db").unwrap();

                    let basics_tree = db.open_tree(b"title.basics.tsv.gz").unwrap();
                    let watched_tree = db.open_tree(b"watched").unwrap();

                    if basics_tree.contains_key(id.as_bytes()).unwrap() {
                        watched_tree.insert(id.as_bytes(), id.as_bytes()).unwrap();
                    }
                }
                _ => {}
            };
        }
        _ => {}
    };
}

fn watched_list_handler() {
    let db = sled::open("my_db").unwrap();

    let watched_tree = db.open_tree(b"watched").unwrap();

    for watched in watched_tree.iter() {
        println!("{}", std::str::from_utf8(&*watched.unwrap().0).unwrap());
    }
}

fn info_handler(id: &str) {
    let db = sled::open("my_db").unwrap();

    let basics_tree = db.open_tree(b"title.basics.tsv.gz").unwrap();
    let result = basics_tree.get(id.as_bytes());
    match result {
        Ok(Some(value)) => {
            let value: Title = serde_json::from_slice(&value).unwrap();
            println!("{:?}", value);
        }
        _ => {}
    }

    let episode_tree = db.open_tree(b"title.episode.tsv.gz").unwrap();
    let result = episode_tree.get(id.as_bytes());
    match result {
        Ok(Some(value)) => {
            let value: Episode = serde_json::from_slice(&value).unwrap();
            println!("{:?}", value);
        }
        _ => {}
    }

    let rating_tree = db.open_tree(b"title.ratings.tsv.gz").unwrap();
    let result = rating_tree.get(id.as_bytes());
    match result {
        Ok(Some(value)) => {
            let value: Rating = serde_json::from_slice(&value).unwrap();
            println!("{:?}", value);
        }
        _ => {}
    }
}

/*
fn update_handler() {
    let db = sled::open("my_db").unwrap();

    let multibar = Arc::new(MultiProgress::new());

    const BASE_URL: &str = "https://datasets.imdbws.com/";

    let db_basics = db.clone();
    let pb_basics = multibar.add(ProgressBar::new(1));
    let handle_basics = thread::spawn(async move {
        let file_name = "title.basics.tsv.gz";

        let url = format!("{}{}", BASE_URL, file_name);

        let client = reqwest::blocking::Client::new();

        let download_size = file::get_download_size(file::File::TitleBasics);

        pb_basics.set_style(
                    ProgressStyle::default_bar()
                        .template("[{bar:40.cyan/blue}] {bytes}/{total_bytes} {prefix} | {msg}")
                        .progress_chars("#>-"),
                );

        pb_basics.set_prefix(&file_name);
        pb_basics.set_length(download_size);
        pb_basics.set_message("Downloading");
        pb_basics.tick();

        let request = client.get(url.as_str());

        let mut out = bytes::BytesMut::with_capacity(download_size as usize);

        let mut download = request.send().await.unwrap();

        while let Some(chunk) = download.chunk().await.unwrap() {
            pb_basics.inc(chunk.len() as u64);
            out.extend(&chunk);
        }

        pb_basics.set_length(1);
        pb_basics.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.cyan/blue}] {pos}/{len} {prefix} | {msg}")
                .progress_chars("#>-"),
        );
        pb_basics.reset();
        pb_basics.set_message("Calculating hash");
        sleep(Duration::from_millis(100)); // workaround
        pb_basics.tick();
        let mut sha256 = Sha256::new();
        sha256.update(&out);
        let hash = sha256.finalize();
        pb_basics.inc(1);

        pb_basics.set_length(1);
        pb_basics.reset();
        pb_basics.set_message("Getting last update info");
        sleep(Duration::from_millis(100)); // workaround
        pb_basics.tick();

        let hash_str = format!("{:x}", hash); // TODO fix:?

        let update_tree = db_basics.open_tree("updates").unwrap();

        let should_update = match update_tree.get(file_name.as_bytes()).unwrap()
        {
            Some(bytes) => {
                let last_hash = String::from(str::from_utf8(&*bytes).unwrap());
                !last_hash.eq(&hash_str)
            }
            None => true,
        };

        if !should_update {
            pb_basics.finish_with_message("Data is up-to-date");
            return;
        }

        pb_basics.inc(1);

        pb_basics.set_length(1);
        pb_basics.reset();
        pb_basics.set_message("Updating data. This may take a while");
        sleep(Duration::from_millis(100)); // workaround
        pb_basics.tick();

        let basics_tree = db_basics.open_tree(file_name.as_bytes()).unwrap();
        basics_tree.clear().unwrap();

        let gz = GzDecoder::new(&*out);

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .flexible(true)
            .from_reader(gz);

        let max_batch_size: usize = 8192;
        let mut current_batch_size: usize = 0;
        let mut batch = Batch::default();

        for result in reader.records() {
            let record = result.unwrap();

            let mut value = vec![];

            let end_year: Option<String> = match &record[6] {
                "\\N" => None,
                value => Some(String::from(value)),
            };

            let runtime_minutes: Option<i32> = match record[7].parse::<i32>() {
                Ok(value) => Some(value),
                _ => None,
            };

            let genres: Option<String> = match record.get(8) {
                Some("\\N") => None,
                Some(value) => Some(String::from(value)),
                None => None,
            };

            value.extend_from_slice(
                serde_json::to_string(&Title {
                    id: String::from(record.get(0).unwrap()),
                    title_type: String::from(record.get(1).unwrap()),
                    primary_title: String::from(record.get(2).unwrap()),
                    original_title: String::from(record.get(3).unwrap()),
                    start_year: String::from(record.get(5).unwrap()),
                    end_year,
                    runtime_minutes,
                    genres,
                })
                .unwrap()
                .as_bytes(),
            );

            batch.insert(record.get(0).unwrap().as_bytes(), value);
            current_batch_size += 1;

            if current_batch_size == max_batch_size {
                basics_tree.apply_batch(batch).unwrap();
                current_batch_size = 0;
                batch = Batch::default();
            }
        }

        if current_batch_size > 0 {
            basics_tree.apply_batch(batch).unwrap();
        }

        update_tree
            .insert(file_name.as_bytes(), hash_str.as_bytes())
            .unwrap();

        pb_basics.inc(1);

        pb_basics.finish_with_message("Data is updated");
    });

    let db_episode = db.clone();
    let pb_episode = multibar.add(ProgressBar::new(1));
    let handle_episode = tokio::spawn(async move {
        let file_name = "title.episode.tsv.gz";

        let url = format!("{}{}", BASE_URL, file_name);

        let client = Client::new();

        let download_size = {
            let resp = client.head(url.as_str()).send().await.unwrap();
            if resp.status().is_success() {
                resp.headers() // Gives us the HeaderMap
                    .get(header::CONTENT_LENGTH) // Gives us an Option containing the HeaderValue
                    .and_then(|ct_len| ct_len.to_str().ok()) // Unwraps the Option as &str
                    .and_then(|ct_len| ct_len.parse().ok()) // Parses the Option as u64
                    .unwrap_or(0) // Fallback to 0
            } else {
                // We return an Error if something goes wrong here
                panic!(format!(
                    "Couldn't download URL: {}. Error: {:?}",
                    url,
                    resp.status(),
                )
                .as_str());
            }
        };

        pb_episode.set_style(
                ProgressStyle::default_bar()
                    .template("[{bar:40.cyan/blue}] {bytes}/{total_bytes} {prefix} | {msg}")
                    .progress_chars("#>-"),
            );

        pb_episode.set_prefix(&file_name);
        pb_episode.set_length(download_size);
        pb_episode.set_message("Downloading");
        pb_episode.tick();

        let request = client.get(url.as_str());

        let mut out = bytes::BytesMut::with_capacity(download_size as usize);

        let mut download = request.send().await.unwrap();

        while let Some(chunk) = download.chunk().await.unwrap() {
            pb_episode.inc(chunk.len() as u64);
            out.extend(&chunk);
        }

        pb_episode.set_length(1);
        pb_episode.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.cyan/blue}] {pos}/{len} {prefix} | {msg}")
                .progress_chars("#>-"),
        );
        pb_episode.reset();
        pb_episode.set_message("Calculating hash");
        sleep(Duration::from_millis(100)); // workaround
        pb_episode.tick();
        let mut sha256 = Sha256::new();
        sha256.update(&out);
        let hash = sha256.finalize();
        pb_episode.inc(1);

        pb_episode.set_length(1);
        pb_episode.reset();
        pb_episode.set_message("Getting last update info");
        sleep(Duration::from_millis(100)); // workaround
        pb_episode.tick();

        let hash_str = format!("{:x}", hash); // TODO fix:?

        let update_tree = db_episode.open_tree("updates").unwrap();

        let should_update = match update_tree.get(file_name.as_bytes()).unwrap()
        {
            Some(bytes) => {
                let last_hash = String::from(str::from_utf8(&*bytes).unwrap());
                !last_hash.eq(&hash_str)
            }
            None => true,
        };

        if !should_update {
            pb_episode.finish_with_message("Data is up-to-date");
            return;
        }

        pb_episode.inc(1);

        pb_episode.set_length(1);
        pb_episode.reset();
        pb_episode.set_message("Updating data. This may take a while");
        sleep(Duration::from_millis(100)); // workaround
        pb_episode.tick();

        let episode_tree = db_episode.open_tree(file_name.as_bytes()).unwrap();
        episode_tree.clear().unwrap();

        let gz = GzDecoder::new(&*out);

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(gz);

        let max_batch_size: usize = 8192;
        let mut current_batch_size: usize = 0;
        let mut batch = Batch::default();

        for result in reader.records() {
            let record = result.unwrap();

            if record.get(2).unwrap() != "\\N" {
                let mut value = vec![];

                value.extend_from_slice(
                    serde_json::to_string(&Episode {
                        id: String::from(record.get(0).unwrap()),
                        parent_id: String::from(record.get(1).unwrap()),
                        season: record.get(2).unwrap().parse::<i32>().unwrap(),
                        episode: record.get(3).unwrap().parse::<i32>().unwrap(),
                    })
                    .unwrap()
                    .as_bytes(),
                );

                batch.insert(record.get(0).unwrap().as_bytes(), value);
                current_batch_size += 1;
            }

            if current_batch_size == max_batch_size {
                episode_tree.apply_batch(batch).unwrap();
                current_batch_size = 0;
                batch = Batch::default();
            }
        }

        if current_batch_size > 0 {
            episode_tree.apply_batch(batch).unwrap();
        }

        update_tree
            .insert(file_name.as_bytes(), hash_str.as_bytes())
            .unwrap();

        pb_episode.inc(1);

        pb_episode.finish_with_message("Data is updated");
    });

    let db_ratings = db.clone();
    let pb_ratings = multibar.add(ProgressBar::new(1));
    let handle_ratings = tokio::spawn(async move {
        let file_name = "title.ratings.tsv.gz";

        let url = format!("{}{}", BASE_URL, file_name);

        let client = Client::new();

        let download_size = {
            let resp = client.head(url.as_str()).send().await.unwrap();
            if resp.status().is_success() {
                resp.headers() // Gives us the HeaderMap
                    .get(header::CONTENT_LENGTH) // Gives us an Option containing the HeaderValue
                    .and_then(|ct_len| ct_len.to_str().ok()) // Unwraps the Option as &str
                    .and_then(|ct_len| ct_len.parse().ok()) // Parses the Option as u64
                    .unwrap_or(0) // Fallback to 0
            } else {
                // We return an Error if something goes wrong here
                panic!(format!(
                    "Couldn't download URL: {}. Error: {:?}",
                    url,
                    resp.status(),
                )
                .as_str());
            }
        };

        pb_ratings.set_style(
                ProgressStyle::default_bar()
                    .template("[{bar:40.cyan/blue}] {bytes}/{total_bytes} {prefix} | {msg}")
                    .progress_chars("#>-"),
            );

        pb_ratings.set_prefix(&file_name);
        pb_ratings.set_length(download_size);
        pb_ratings.set_message("Downloading");
        pb_ratings.tick();

        let request = client.get(url.as_str());

        let mut out = bytes::BytesMut::with_capacity(download_size as usize);

        let mut download = request.send().await.unwrap();

        while let Some(chunk) = download.chunk().await.unwrap() {
            pb_ratings.inc(chunk.len() as u64);
            out.extend(&chunk);
        }

        pb_ratings.set_length(1);
        pb_ratings.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.cyan/blue}] {pos}/{len} {prefix} | {msg}")
                .progress_chars("#>-"),
        );
        pb_ratings.reset();
        pb_ratings.set_message("Calculating hash");
        sleep(Duration::from_millis(100)); // workaround
        pb_ratings.tick();
        let mut sha256 = Sha256::new();
        sha256.update(&out);
        let hash = sha256.finalize();
        pb_ratings.inc(1);

        pb_ratings.set_length(1);
        pb_ratings.reset();
        pb_ratings.set_message("Getting last update info");
        sleep(Duration::from_millis(100)); // workaround
        pb_ratings.tick();

        let hash_str = format!("{:x}", hash); // TODO fix:?

        let update_tree = db_ratings.open_tree("updates").unwrap();

        let should_update = match update_tree.get(file_name.as_bytes()).unwrap()
        {
            Some(bytes) => {
                let last_hash = String::from(str::from_utf8(&*bytes).unwrap());
                !last_hash.eq(&hash_str)
            }
            None => true,
        };

        if !should_update {
            pb_ratings.finish_with_message("Data is up-to-date");
            return;
        }

        pb_ratings.inc(1);

        pb_ratings.set_length(1);
        pb_ratings.reset();
        pb_ratings.set_message("Updating data. This may take a while");
        sleep(Duration::from_millis(100)); // workaround
        pb_ratings.tick();

        let episode_tree = db_ratings.open_tree(file_name.as_bytes()).unwrap();
        episode_tree.clear().unwrap();

        let gz = GzDecoder::new(&*out);

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(gz);

        let max_batch_size: usize = 8192;
        let mut current_batch_size: usize = 0;
        let mut batch = Batch::default();

        for result in reader.records() {
            let record = result.unwrap();

            if record.get(2).unwrap() != "\\N" {
                let mut value = vec![];

                value.extend_from_slice(
                    serde_json::to_string(&Rating {
                        id: String::from(record.get(0).unwrap()),
                        avarage_rating: record
                            .get(1)
                            .unwrap()
                            .parse::<f32>()
                            .unwrap(),
                        num_votes: record
                            .get(2)
                            .unwrap()
                            .parse::<i32>()
                            .unwrap(),
                    })
                    .unwrap()
                    .as_bytes(),
                );

                batch.insert(record.get(0).unwrap().as_bytes(), value);
                current_batch_size += 1;
            }

            if current_batch_size == max_batch_size {
                episode_tree.apply_batch(batch).unwrap();
                current_batch_size = 0;
                batch = Batch::default();
            }
        }

        if current_batch_size > 0 {
            episode_tree.apply_batch(batch).unwrap();
        }

        update_tree
            .insert(file_name.as_bytes(), hash_str.as_bytes())
            .unwrap();

        pb_ratings.inc(1);

        pb_ratings.finish_with_message("Data is updated");
    });

    let multibar = {
        let multibar = multibar.clone();

        tokio::task::spawn_blocking(move || multibar.join())
    };

    let handles = vec![handle_basics, handle_ratings, handle_episode];
    join_all(handles).await;

    multibar.await.unwrap().unwrap();
}
*/
