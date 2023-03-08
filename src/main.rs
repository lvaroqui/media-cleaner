mod arr;
mod config;
mod media_item;
mod overseerr;
mod shared;
mod tautulli;
mod tmdb;

use std::{io, process::Command};

use color_eyre::Result;

use config::Config;
use dialoguer::MultiSelect;
use media_item::get_requests_data;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    if let Err(err) = Config::read_conf() {
        eprintln!("Error reading config: {}", err);
        std::process::exit(1);
    }

    let mut requests = get_requests_data().await?;

    clear_screen()?;

    let chosen: Vec<usize> = MultiSelect::new()
        .with_prompt("Choose what media to delete.")
        .max_length(5)
        .items(&requests)
        .interact()?;

    if chosen.len() == 0 {
        println!("No items selected. Exiting...");
        return Ok(());
    }

    clear_screen()?;

    println!("Are you sure you want to delete the following items (y/n):");
    chosen.iter().for_each(|selection| {
        if let Some(media_item) = requests.get(*selection) {
            let default = "Unknown".to_string();
            let title = media_item.get_title().as_ref().unwrap_or(&default);
            let media_type = media_item.get_media_type();
            println!("- {} - {}", title, media_type.to_string());
        } else {
            println!("- Unknown item");
        }
    });

    let user_input = get_user_input()?;

    if !user_input.starts_with("y") {
        println!("Cancelling...");
        return Ok(());
    }

    for selection in chosen.into_iter().rev() {
        let media_item = requests.swap_remove(selection);
        media_item.delete_item().await?;
    }

    Ok(())
}

fn clear_screen() -> Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg("cls").status()?;
        Ok(())
    } else {
        Command::new("clear").status()?;
        Ok(())
    }
}

fn get_user_input() -> Result<String> {
    let mut user_input = String::new();
    let stdin = io::stdin();

    stdin.read_line(&mut user_input)?;

    Ok(user_input.to_lowercase())
}
