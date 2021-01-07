use std::path;
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;
use std::process;

mod error;
mod webhook;
use crate::error::Error;
use crate::error::Result;
use crate::webhook::Webhook;
use crate::webhook::EmbedBuilder;

use scraper::{ElementRef, Html, Selector};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
static APP_STATE_DIRECTORY: &str = "/etc/tarjousbot";

fn get_webhook_url() -> Result<String> {
    let webhook_url_path = Path::new(APP_STATE_DIRECTORY).join("webhook.conf");
    let mut s = String::new();
    File::open(webhook_url_path)?.read_to_string(&mut s)?;
    Ok(s)
}

fn try_read_u32(path: path::PathBuf) -> Result<Option<u32>> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(err) => {
            if let io::ErrorKind::NotFound = err.kind() {
                return Ok(None)
            } else {
                return Err(err.into())
            }
        },
    };
    Ok(file.read_u32::<LittleEndian>().ok())
}

fn write_u32(path: path::PathBuf, u: u32) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_u32::<LittleEndian>(u)?;
    Ok(())
}

fn get_last_page() -> Result<Option<u32>> {
    let last_page_path = Path::new(APP_STATE_DIRECTORY).join("last_page");
    try_read_u32(last_page_path)
}

fn set_last_page(page: u32) -> Result<()> {
    let last_page_path = Path::new(APP_STATE_DIRECTORY).join("last_page");
    write_u32(last_page_path, page)
}

fn get_last_sent_post() -> Result<Option<u32>> {
    let last_post_path = Path::new(APP_STATE_DIRECTORY).join("last_post");
    try_read_u32(last_post_path)
}

fn set_last_sent_post(post: u32) -> Result<()> {
    let last_page_path = Path::new(APP_STATE_DIRECTORY).join("last_post");
    write_u32(last_page_path, post)
}

fn get_page_url(page: u32) -> String {
    format!("https://bbs.io-tech.fi/threads/151/page-{}", page)
}

fn get_post_id(post: ElementRef) -> Result<u32> {
    post.value()
        .attr("data-content")
        .ok_or(Error::Scraping)?
        .strip_prefix("post-")
        .ok_or(Error::Scraping)?
        .parse()
        .or(Err(Error::Scraping))
}

fn run() -> Result<()> {
    let mut page_number = get_last_page()?.unwrap_or(u32::MAX);
    let last_sent_post = get_last_sent_post()?;

    let client = reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;
    let webhook = Webhook::with_client(&client);
    let webhook_url = get_webhook_url()?;

    let post_selector = Selector::parse(".message").unwrap();
    let next_page_selector = Selector::parse(".pageNav-page--current+ .pageNav-page").unwrap();

    let time_selector = Selector::parse(".u-dt").unwrap();
    let username_selector = Selector::parse(".username").unwrap();
    let avatar_selector = Selector::parse(".avatar img").unwrap();
    let content_selector = Selector::parse(".bbWrapper").unwrap();

    let mut last_id;

    loop {
        eprintln!("Get page {}", page_number);
        let response = client
            .get(&get_page_url(page_number))
            .send()?
            .error_for_status()?;
        if page_number == u32::MAX {
            // figure out the actual page from the url
            page_number = response
                .url()
                .path_segments()
                .ok_or(Error::Scraping)?
                .last()
                .ok_or(Error::Scraping)?
                .strip_prefix("page-")
                .ok_or(Error::Scraping)?
                .parse()
                .or(Err(Error::Scraping))?;
        }

        let body = response.text()?;
        let fragment = Html::parse_document(&body);

        let posts = fragment.select(&post_selector);

        if let Some(last_sent_id) = last_sent_post {
            let mut last_id_temp = last_sent_id;

            for post in posts {
                let post_id = get_post_id(post)?;
                if post_id > last_sent_id {
                    eprintln!("New message: id {}", post_id);

                    let timestamp = post
                        .select(&time_selector)
                        .next()
                        .ok_or(Error::Scraping)?
                        .value()
                        .attr("datetime")
                        .ok_or(Error::Scraping)?;

                    let username_element = post
                        .select(&username_selector)
                        .next()
                        .ok_or(Error::Scraping)?;
                    let username = username_element
                        .text()
                        .next()
                        .ok_or(Error::Scraping)?;
                    let user_url = format!("https://bbs.io-tech.fi{}", username_element
                        .value()
                        .attr("href")
                        .ok_or(Error::Scraping)?
                    );
                    
                    let avatar_url = post
                        .select(&avatar_selector)
                        .next()
                        .map(|element| element
                            .value()
                            .attr("src")
                            .ok_or(Error::Scraping)
                            .map(|s| format!("https://bbs.io-tech.fi{}", s))
                        )
                        .transpose()?;

                    let content: String = post
                        .select(&content_selector)
                        .next()
                        .ok_or(Error::Scraping)?
                        .children()
                        .map(|child| {
                            match child.value() {
                                scraper::Node::Text(text) => text,
                                scraper::Node::Element(element) => {
                                    match element.name() {
                                        "br" => "\n",
                                        "a" => element.attr("href").unwrap_or(""),
                                        _ => {
                                            ElementRef::wrap(child)
                                                .unwrap()
                                                .text()
                                                .next()
                                                .unwrap_or("")
                                        },
                                    }
                                },
                                _ => "",
                            }
                        })
                        .collect();
                    let default_title = "Uusi tarjous";
                    let title = content
                        .strip_prefix("Tuote:")
                        .unwrap_or(default_title)
                        .split("\n")
                        .next()
                        .unwrap_or(default_title);

                    eprintln!("Username: {}, Title: {}, Content: {}", username, title, content);
                    webhook
                        .execute(&webhook_url)
                        .embed(EmbedBuilder::new()
                            .timestamp(timestamp)
                            .author(Some(username), Some(&user_url), avatar_url.as_deref())
                            .description(&content)
                            .title(title)
                        )
                        .send()?.error_for_status()?;

                    last_id_temp = post_id
                }
            }
            last_id = last_id_temp;
        } else {
            last_id = get_post_id(posts.last().ok_or(Error::Scraping)?)?;
        }

        match fragment.select(&next_page_selector).next() {
            Some(next_page) => {
                page_number = next_page
                    .text()
                    .next()
                    .ok_or(Error::Scraping)?
                    .parse()
                    .or(Err(Error::Scraping))?;
            }
            None => break,
        }
    }

    set_last_page(page_number)?;
    set_last_sent_post(last_id)?;

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}