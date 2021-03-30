#![allow(dead_code)]

use reqwest::blocking::Client;
use serde::Serialize;

pub struct Webhook<'a> {
    client: &'a Client,
}

impl<'a> Webhook<'a> {
    pub fn with_client(client: &'a Client) -> Self {
        Self { client }
    }
}

#[derive(Serialize, Default)]
struct EmbedFooter<'a> {
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<&'a str>,
}

#[derive(Serialize)]
struct EmbedImage<'a> {
    url: &'a str,
}

#[derive(Serialize)]
struct EmbedThumbnail<'a> {
    url: &'a str,
}

#[derive(Serialize, Default)]
struct EmbedAuthor<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<&'a str>,
}

#[derive(Serialize, Default)]
struct EmbedField<'a> {
    name: &'a str,
    value: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    inline: Option<bool>,
}

#[derive(Serialize, Default)]
struct Embed<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<EmbedFooter<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<EmbedImage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<EmbedThumbnail<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<EmbedAuthor<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fields: Vec<EmbedField<'a>>,
}

#[derive(Serialize, Default)]
struct ExecuteWebhook<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    embeds: Vec<&'a Embed<'a>>,
}

pub struct EmbedBuilder<'a> {
    embed: Embed<'a>,
}

impl<'a> EmbedBuilder<'a> {
    pub fn new() -> Self {
        Self {
            embed: Embed::default(),
        }
    }

    pub fn title(&mut self, title: &'a str) -> &mut Self {
        self.embed.title = Some(title);
        self
    }

    pub fn description(&mut self, description: &'a str) -> &mut Self {
        self.embed.description = Some(description);
        self
    }

    pub fn url(&mut self, url: &'a str) -> &mut Self {
        self.embed.url = Some(url);
        self
    }

    pub fn timestamp(&mut self, timestamp: &'a str) -> &mut Self {
        self.embed.timestamp = Some(timestamp);
        self
    }

    pub fn color(&mut self, color: i32) -> &mut Self {
        self.embed.color = Some(color);
        self
    }

    pub fn footer(&mut self, text: &'a str, icon_url: Option<&'a str>) -> &mut Self {
        self.embed.footer = Some(EmbedFooter { text, icon_url });
        self
    }

    pub fn image(&mut self, url: &'a str) -> &mut Self {
        self.embed.image = Some(EmbedImage { url });
        self
    }

    pub fn thumbnail(&mut self, url: &'a str) -> &mut Self {
        self.embed.thumbnail = Some(EmbedThumbnail { url });
        self
    }

    pub fn author(
        &mut self,
        name: Option<&'a str>,
        url: Option<&'a str>,
        icon_url: Option<&'a str>,
    ) -> &mut Self {
        self.embed.author = Some(EmbedAuthor {
            name,
            url,
            icon_url,
        });
        self
    }

    pub fn field(&mut self, name: &'a str, value: &'a str, inline: Option<bool>) -> &mut Self {
        self.embed.fields.push(EmbedField {
            name,
            value,
            inline,
        });
        self
    }
}

pub struct ExecutionBuilder<'a> {
    webhook: &'a Webhook<'a>,
    url: &'a str,
    payload: ExecuteWebhook<'a>,
}

impl<'a> ExecutionBuilder<'a> {
    pub fn content(&mut self, content: &'a str) -> &mut Self {
        self.payload.content = Some(content);
        self
    }

    pub fn username(&mut self, username: &'a str) -> &mut Self {
        self.payload.username = Some(username);
        self
    }

    pub fn avatar_url(&mut self, avatar_url: &'a str) -> &mut Self {
        self.payload.avatar_url = Some(avatar_url);
        self
    }

    pub fn tts(&mut self, tts: bool) -> &mut Self {
        self.payload.tts = Some(tts);
        self
    }

    pub fn file(&mut self, file: &'a str) -> &mut Self {
        self.payload.file = Some(file);
        self
    }

    pub fn embed(&mut self, embed: &'a EmbedBuilder) -> &mut Self {
        self.payload.embeds.push(&embed.embed);
        self
    }

    pub fn send(&self) -> reqwest::Result<reqwest::blocking::Response> {
        self.webhook
            .client
            .post(self.url)
            .json(&self.payload)
            .send()
    }
}

impl<'a> Webhook<'a> {
    pub fn execute(&'a self, url: &'a str) -> ExecutionBuilder<'a> {
        ExecutionBuilder {
            webhook: &self,
            url,
            payload: ExecuteWebhook::default(),
        }
    }
}
