use anyhow::{Context, Result};
use clap::{App, Arg, SubCommand};
use dirs;
use glob::glob;
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::thread;

#[derive(Deserialize, Serialize)]
struct Config {
    dirs: Vec<String>,
    duration: Option<String>,
    active_dir: Option<usize>,
    current: Option<String>,
    next: Option<Vec<String>>,
}

/// Open a given config file and try to parse the contents into a Config struct
fn parse_config(fname: &String) -> Result<Config> {
    let config = std::fs::read_to_string(&fname)
        .with_context(|| format!("Could not open wallch config: {}", fname))?;

    let mut config: Config =
        toml::from_str(&config).with_context(|| format!("Could not parse user config"))?;

    // Duration may be unspecified; use 10 minutes as default
    let duration = config.duration.unwrap_or(String::from("10m"));

    // The active directory may be unspecified; use the first directory in `dirs` as default
    let active_dir = config.active_dir.unwrap_or(0);

    // Save duration and active directory to the config
    config.duration = Some(duration);
    config.active_dir = Some(active_dir);

    Ok(config)
}

/// Saves a given config to a file
fn write_config(config: &Config, fname: &String) -> Result<()> {
    let conf_str =
        toml::to_string(config).with_context(|| format!("Could not serialize config"))?;

    std::fs::write(fname, conf_str).with_context(|| format!("Could not write config to file"))?;

    Ok(())
}

/// Randomly select a new wallpaper from the given directory
fn select_new(dir: &String, rng: &mut ThreadRng) -> Result<String> {
    let imgs = glob(format!("{}/*.*", dir).as_str())
        .with_context(|| format!("Could not read dir: {}", dir))?;

    let img = imgs
        .choose(rng)
        .with_context(|| format!("Could not pick image from dir: {}", dir))?;

    Ok(format!("file://{}", img.unwrap().display()))
}

/// Select the next wallpaper for each directory and "cache" it (i.e. store it in config)
fn cache_next(config_str: &String, rng: &mut ThreadRng) -> Result<()> {
    let mut config = parse_config(&config_str)?;

    let mut next: Vec<String> = Vec::new();
    for dir in &config.dirs {
        next.push(select_new(&dir, rng)?);
    }

    config.next = Some(next);
    write_config(&config, config_str)?;

    Ok(())
}

/// Get the next cached wallpaper from the given directory or select a new one
fn get_next(config_str: &String, rng: &mut ThreadRng) -> Result<String> {
    let config = parse_config(&config_str)?;
    let active_dir = config.active_dir.unwrap_or(0);

    if let Some(next) = config.next {
        // Next wallpaper has been pre-set; return it
        Ok(String::from(&next[active_dir]))
    } else {
        // No pre-set next wallpaper; select one on the fly
        Ok(select_new(&config.dirs[active_dir], rng)?)
    }
}

/// Set the wallpaper to a given file
fn set_wallpaper(fname: &String) -> Result<()> {
    std::process::Command::new("gsettings")
        .args(&["set", "org.gnome.desktop.background", "picture-uri", &fname])
        .status()
        .with_context(|| "Could not set desktop background")?;

    Ok(())
}

/// Perform one iteration of the change-wallpaper-and-sleep cycle
fn run(config_str: &String, rng: &mut ThreadRng) -> Result<()> {
    // We re-read the config in every loop iteration so it can be changed on the fly
    let mut config = parse_config(&config_str)?;

    // Get or select the next WP
    let current = get_next(&config_str, rng)?;

    // Set it
    set_wallpaper(&current)?;

    // Save it to config
    config.current = Some(current);
    write_config(&config, config_str)?;

    // Pre-select the next WP
    cache_next(&config_str, rng)?;

    // Wait for next cycle
    let duration = config.duration.unwrap_or(String::from("10m"));
    let duration = humanize_rs::duration::parse(&duration)
        .with_context(|| format!("Could not parse duration"))?;

    thread::sleep(duration);

    Ok(())
}

/// Choose and apply a new random wallpaper
fn next(config_str: &String, rng: &mut ThreadRng) -> Result<()> {
    let mut config = parse_config(&config_str)?;

    let current = get_next(config_str, rng)?;
    set_wallpaper(&current)?;

    config.current = Some(current);
    write_config(&config, config_str)?;

    cache_next(&config_str, rng)?;

    Ok(())
}

/// Switch to the next directory in the dirs list and apply a new wallpaper from it
fn toggle(config_str: &String, rng: &mut ThreadRng) -> Result<()> {
    let mut config = parse_config(&config_str)?;
    let active_dir = config.active_dir.unwrap_or(0);

    // Switch active dir to next in dirs list, wrapping around
    config.active_dir = if active_dir + 1 < config.dirs.len() {
        Some(active_dir + 1)
    } else {
        Some(0)
    };

    // Save the new config
    write_config(&config, &config_str)?;

    // Choose a new wallpaper from the new dir
    next(&config_str, rng)?;

    Ok(())
}

/// Return the path of the current wallpaper file, stripped of the "file://" prefix
fn current(config: &Config) -> Option<String> {
    let current = &config.current;

    match current {
        Some(s) => Some(s.replace("file://", "")),
        _ => None,
    }
}

fn main() -> Result<()> {
    let matches = App::new("GNOME Wallpape-rs")
        .version("0.1.0")
        .author("Jakob Pfender <jakob.pfender@gmail.com>")
        .about("Wallpaper switcher for GNOME")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG")
                .help("Specify user config")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("duration")
                .short("d")
                .long("duration")
                .value_name("LENGTH")
                .help("Set/change wallpaper duration")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("active")
                .short("a")
                .long("active")
                .value_name("ACTIVE_DIR")
                .help("Set active wallpaper directory (index of dirs vector)")
                .takes_value(true),
        )
        .subcommand(SubCommand::with_name("run").about("Starts the wallpaper changer loop"))
        .subcommand(SubCommand::with_name("next").about("Change to a new wallpaper"))
        .subcommand(
            SubCommand::with_name("toggle")
                .about("Change wallpaper directory and apply a new wallpaper"),
        )
        .subcommand(SubCommand::with_name("current").about("Print current wallpaper path"))
        .get_matches();

    let home_dir = dirs::home_dir().unwrap().to_str().unwrap().to_string();

    // Config file needs to be present in home dir or supplied via command line
    let config_str = String::from(
        matches
            .value_of("config")
            .unwrap_or(&format!("{}/wallch.toml", home_dir)),
    );
    let mut config = parse_config(&config_str)?;

    // Use user-specified duration if present
    if let Some(d) = matches.value_of("duration") {
        config.duration = Some(String::from(d));
    }

    // Use user-specified active directory if present
    if let Some(a) = matches.value_of("active") {
        config.active_dir = Some(a.parse()?);
    }

    // Config has been updated with values for all optional parameters; save it back to file
    write_config(&config, &config_str)?;

    let mut rng = rand::thread_rng();

    if let Some(_) = matches.subcommand_matches("run") {
        loop {
            run(&config_str, &mut rng)?;
        }
    } else if let Some(_) = matches.subcommand_matches("next") {
        next(&config_str, &mut rng)?;
    } else if let Some(_) = matches.subcommand_matches("toggle") {
        toggle(&config_str, &mut rng)?;
    } else if let Some(_) = matches.subcommand_matches("current") {
        println!("{}", current(&config).unwrap_or(String::new()));
    }

    Ok(())
}
