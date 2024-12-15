use core::panic;
use std::{env::var, path::Path};
use bsky_sdk::{agent::config::{Config, FileStore}, api::{self, app::bsky::feed::post}, BskyAgent};

use api::types::string::Datetime;
use chrono_tz::Europe::Amsterdam;
use cron_tab::AsyncCron;
use crate::images::Images;

mod images;

async fn get_agent() -> BskyAgent {
    if Path::new("./config.json").exists() {
        let agent = BskyAgent::builder()
            .config(Config::load(&FileStore::new("config.json")).await.unwrap())
            .build()
            .await;

        if agent.is_ok() {
            return agent.unwrap()
        }
    }

    let email = match var("BSKY_EMAIL") {
        Ok(email) => email,
        Err(err) => panic!("'BSKY_EMAIL' is not set: {}", err)
    };

    let pass = match var("BSKY_PASS") {
        Ok(pass) => pass,
        Err(err) => panic!("'BSKY_PASS' is not set: {}", err)
    };

    let agent = BskyAgent::builder().build().await.unwrap();
    let login = agent.login(email, pass).await;
    println!("Logging in.");

    match login {
        Ok(_) => (),
        Err(err) => {
            panic!("Error while login. {}", err)
        }
    }

    if let Err(err) = agent
        .to_config()
        .await
        .save(&FileStore::new("config.json"))
        .await {
        panic!("Error while saving session to config.json: {}", err)
    };

    agent
}

#[tokio::main]
async fn main() {
    let agent = get_agent().await;
    let images = Images::default();

    let mut cron = AsyncCron::new(Amsterdam);

    let _first_job_id = cron.add_fn("0 13 * * * *", move || {
        let agent = agent.clone();
        let mut images = images.clone();

        async move {
            post(agent, &mut images).await;
        }
    }).await;

    cron.start().await;
    tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
}

async fn post(agent: BskyAgent, images: &mut Images) {
    let image = images.get_random_image();

    if image.is_none() {
        return
    }

    let image = image.unwrap();

    let embed = image.make_into_embed(&agent).await;

    let record = agent
        .create_record(post::RecordData {
            created_at: Datetime::now(),
            embed: embed,
            entities: None,
            facets: None,
            labels: None,
            langs: None,
            reply: None,
            tags: None,
            text: "".to_string()
        })
        .await;

    if record.is_err() {
        println!("Failed to post image: {}", record.unwrap_err())
    }

    images.move_image(&image);
}