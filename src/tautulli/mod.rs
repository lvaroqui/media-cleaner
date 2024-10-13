mod api;
mod responses;

use chrono::prelude::*;
use color_eyre::{owo_colors::OwoColorize, Result};
use serde::de::DeserializeOwned;
use std::{collections::BTreeMap, fmt::Display};

use self::responses::{History, HistoryItem, HistoryMovieItem};
use crate::{shared::MediaType, tautulli::responses::ResponseObj};

#[derive(Debug)]
pub enum WatchHistory {
    Movie(ItemWatches<UserMovieWatch>),
    TvShow(ItemWatches<UserEpisodeWatch>),
}

impl WatchHistory {
    fn from_user_watches(
        user_watches: BTreeMap<&String, &HistoryItem>,
        media_type: &MediaType,
        rating_key: &str,
    ) -> Self {
        match media_type {
            MediaType::Movie => WatchHistory::create_movie_history(user_watches, rating_key),
            MediaType::Tv => WatchHistory::create_tv_history(user_watches, rating_key),
        }
    }

    fn create_movie_history(
        user_watches: BTreeMap<&String, &HistoryItem>,
        rating_key: &str,
    ) -> Self {
        let watches = user_watches
            .iter()
            .map(|(user, movie_watch)| UserMovieWatch {
                display_name: user.to_string(),
                last_watched: unix_seconds_to_date(movie_watch.date).unwrap_or_else(|| {
                    panic!("Failed to parse unix time for rating key {}", rating_key)
                }),
                progress: movie_watch.percent_complete.try_into().unwrap_or(100),
            })
            .collect();

        WatchHistory::Movie(watches)
    }

    fn create_tv_history(user_watches: BTreeMap<&String, &HistoryItem>, rating_key: &str) -> Self {
        let watches = user_watches
            .iter()
            .map(|(user, tv_watch)| UserEpisodeWatch {
                display_name: user.to_string(),
                last_watched: unix_seconds_to_date(tv_watch.date).unwrap_or_else(|| {
                    panic!("Failed to parse unix time for rating key {}", rating_key)
                }),
                progress: tv_watch.percent_complete.try_into().unwrap_or(100),
                season: tv_watch.parent_media_index.unwrap(),
                episode: tv_watch.media_index.unwrap(),
            })
            .collect();

        WatchHistory::TvShow(watches)
    }
}

impl Display for WatchHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Movie(watches) => write_watches(f, watches),
            Self::TvShow(watches) => write_watches(f, watches),
        }
    }
}

fn write_watches<T>(f: &mut std::fmt::Formatter, watches: &ItemWatches<T>) -> std::fmt::Result
where
    T: Display,
{
    if !watches.is_empty() {
        write!(f, "Watch history:")?;
        for watch in watches.iter() {
            write!(f, "\n      * {}", watch)?;
        }
        Ok(())
    } else {
        write!(f, "No watch history.")
    }
}

pub type ItemWatches<T> = Vec<T>;

#[derive(Debug)]
pub struct UserEpisodeWatch {
    display_name: String,
    last_watched: DateTime<Utc>,
    progress: u8,
    season: u32,
    episode: u32,
}

impl Display for UserEpisodeWatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Last watch by {}, was at {}. Season {} Episode {}, with {} complete.",
            self.display_name.yellow(),
            self.last_watched.format("%d-%m-%Y").blue(),
            self.season.yellow(),
            self.episode.yellow(),
            format!("{}%", self.progress).blue()
        )
    }
}

#[derive(Debug)]
pub struct UserMovieWatch {
    display_name: String,
    last_watched: DateTime<Utc>,
    progress: u8,
}

impl Display for UserMovieWatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Last watch by {} at {}, with {} progress.",
            self.display_name.yellow(),
            self.last_watched.format("%d-%m-%Y").blue(),
            format!("{}%", self.progress).blue()
        )
    }
}

pub async fn get_item_watches(rating_key: &str, media_type: &MediaType) -> Result<WatchHistory> {
    let history = get_item_history(rating_key, media_type).await?;

    let latest_user_history =
        history
            .iter()
            .fold(BTreeMap::new(), |mut user_latest_watch, current_watch| {
                user_latest_watch
                    .entry(&current_watch.user)
                    .and_modify(|entry: &mut &HistoryItem| {
                        if entry.date < current_watch.date {
                            *entry = current_watch;
                        }
                    })
                    .or_insert(current_watch);

                user_latest_watch
            });

    Ok(WatchHistory::from_user_watches(
        latest_user_history,
        media_type,
        rating_key,
    ))
}

async fn get_item_history(rating_key: &str, media_type: &MediaType) -> Result<Vec<HistoryItem>> {
    if let MediaType::Movie = media_type {
        let history: Vec<HistoryMovieItem> = get_full_history(rating_key, "rating_key").await?;
        Ok(movie_item_to_history_item(history))
    } else {
        let history: Vec<HistoryItem> =
            get_full_history(rating_key, "grandparent_rating_key").await?;
        Ok(history)
    }
}

async fn get_full_history<T>(rating_key: &str, rating_key_kind: &str) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let length = 1000;
    let length_string = length.to_string();
    let mut history: Vec<T> = Vec::new();
    let mut page = 0;
    loop {
        let page_string = page.to_string();
        let params = vec![
            (rating_key_kind, rating_key),
            ("length", &length_string),
            ("start", &page_string),
        ];
        let mut history_page: ResponseObj<History<T>> =
            api::get_obj("get_history", Some(params)).await?;

        if history_page.response.data.data.len() < length {
            history.append(&mut history_page.response.data.data);
            break;
        } else {
            history.append(&mut history_page.response.data.data);
            page += 1;
        }
    }

    Ok(history)
}

fn movie_item_to_history_item(history: Vec<HistoryMovieItem>) -> Vec<HistoryItem> {
    history
        .into_iter()
        .map(|item| HistoryItem {
            user: item.user,
            date: item.date,
            duration: item.duration,
            percent_complete: item.percent_complete,
            media_index: None,
            parent_media_index: None,
        })
        .collect()
}

fn unix_seconds_to_date(unix_seconds: i64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp_millis(unix_seconds * 1000)
}
