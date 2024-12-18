use core::panic;
use std::{env::var, path::Path};
use bsky_sdk::{agent::config::{Config, FileStore}, api::{self, app::bsky::feed::post}, rich_text::RichText, BskyAgent};

use api::types::string::Datetime;
use chrono_tz::Europe::Amsterdam;
use tokio_schedule::{every, Job};
use crate::images::Images;

mod images;

async fn get_agent() -> BskyAgent {
    if Path::new("./login.json").exists() {
        let agent = BskyAgent::builder()
            .config(Config::load(&FileStore::new("login.json")).await.unwrap())
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
            panic!("Error while login: {}", err)
        }
    }

    if let Err(err) = agent
        .to_config()
        .await
        .save(&FileStore::new("login.json"))
        .await {
        panic!("Error while saving session to login.json: {}", err)
    };

    agent
}

#[tokio::main]
async fn main() {
    let agent = get_agent().await;
    let images = Images::default();

    let daily_post = every(1).day().at(13, 00, 00)
        .in_timezone(&Amsterdam)
        .perform(|| async {
            let agent = agent.clone();
            let mut images = images.clone();

            post(agent, &mut images).await;
        });
    
    daily_post.await;
}

async fn post(agent: BskyAgent, images: &mut Images) {
    let image = images.get_random_image();

    if image.is_none() {
        return
    }

    let image = image.unwrap();

    let embed = image.make_into_embed(&agent).await;

    let rt = match &image.credit {
        Some(credit) => {
            RichText::new_with_detect_facets(
                format!("#azumanga #azumangadaioh\n\nCredit: {}", credit),
            ).await
        },
        None => {
            RichText::new_with_detect_facets(
                "#azumanga #azumangadaioh"
            ).await
        }
    }
    .unwrap();

    let record = agent
        .create_record(post::RecordData {
            created_at: Datetime::now(),
            embed: embed,
            entities: None,
            facets: rt.facets,
            labels: None,
            langs: None,
            reply: None,
            tags: None,
            text: rt.text
        })
        .await;

    if record.is_err() {
        println!("Failed to post image: {}", record.unwrap_err())
    }

    images.move_files(&image);
}