use std::time;
use std::thread;
use std::{path::{Path, PathBuf}, process::{Command, Output}, io::Error, env::{args, self}};

use rand::seq::SliceRandom;
use walkdir::WalkDir;
use image::GenericImageView;


enum DEWM {
    Gnome, Sway
}


pub trait ExtensionVecU8ToString {
    fn to_string(self) -> String;
}
impl ExtensionVecU8ToString for Vec<u8> {
    fn to_string(self) -> String {
        String::from_utf8(self).unwrap()
    }
}


fn get_de_wm() -> DEWM {
    let xdg_current_desktop: String = match env::var("XDG_CURRENT_DESKTOP") {
        Ok(val) => { val }
        Err(_e) => { panic!() }
    };
    // println!("xdg_current_desktop = {xdg_current_desktop}");
    match xdg_current_desktop {
        _ if xdg_current_desktop.ends_with("GNOME") => {
            DEWM::Gnome
        }
        _ if xdg_current_desktop.ends_with("SWAY") => {
            DEWM::Sway
        }
        _ => { todo!() }
    }
}

fn set_wallpaper(path: &Path) -> Result<Output, Error> {
    assert!(path.to_str().unwrap().len() > 0);
    assert!(path.is_file());
    match get_de_wm() {
        DEWM::Gnome => {
            let path_str: String = path.display().to_string();
            Command::new("gsettings")
                .env("GSETTINGS_BACKEND", "dconf")
                .arg("set")
                .arg("org.gnome.desktop.background")
                .arg("picture-uri")
                .arg(format!("file://{path_str}"))
                .output()?;
            Command::new("gsettings")
                .env("GSETTINGS_BACKEND", "dconf")
                .arg("set")
                .arg("org.gnome.desktop.background")
                .arg("picture-uri-dark")
                .arg(format!("file://{path_str}"))
                .output()
        }
        DEWM::Sway => {
            todo!()
        }
    }
}

fn calc_image_brightness(path: &PathBuf) -> Option<f32> {
    let image = image::open(path);
    if image.is_err() { return None; }
    let image = image.unwrap();
    let mut brightness: u64 = 0;
    for (_w, _h, pixel) in image.pixels() {
        // brightness += pixel.0[0] as u64; // alpha
        brightness += pixel.0[1] as u64;
        brightness += pixel.0[2] as u64;
        brightness += pixel.0[3] as u64;
    }
    Some((brightness as f64 / (4.0 * 255.0 * image.dimensions().0 as f64 * image.dimensions().1 as f64)) as f32)
}

#[derive(Debug)]
struct Wallpaper {
    path: PathBuf,
    brightness: f32,
}

#[derive(Debug)]
struct Config {
    delay: Option<u32>,
    wallpapers_path: Option<PathBuf>,
    wallpapers: Vec<Wallpaper>,
}
impl Config {
    fn new() -> Self {
        Config {
            delay: None,
            wallpapers_path: None,
            wallpapers: vec![],
        }
    }

    fn init_wallpapers(&mut self) {
        for entry in WalkDir::new(self.wallpapers_path.clone().unwrap()) {
            if entry.as_ref().unwrap().path().is_dir() { continue; }
            println!("{}", entry.as_ref().unwrap().path().display());
            let path: PathBuf = entry.unwrap().path().to_path_buf();
            let brightness: Option<f32> = calc_image_brightness(&path);
            if brightness.is_none() {
                let path_str: String = path.display().to_string();
                println!("Skipping {path_str}");
                continue;
            }
            let brightness: f32 = brightness.unwrap();
            println!("brightness = {brightness}");
            self.wallpapers.push(Wallpaper { path, brightness });
        }
    }
}

fn main() {
    let args: Vec<String> = args().collect::<Vec<String>>()[1..].to_vec();

    const ARG_DELAY_SHORT: &str = "-d=";
    const ARG_DELAY_LONG: &str = "--delay=";

    const ARG_PATH_SHORT: &str = "-p=";
    const ARG_PATH_LONG: &str = "--path=";

    let mut config: Config = Config::new();
    for arg in args {
        match arg {
            arg_delay if arg.starts_with(ARG_DELAY_SHORT) || arg.starts_with(ARG_DELAY_LONG) => {
                let delay_str: String = arg_delay[if arg_delay.starts_with(ARG_DELAY_SHORT) { ARG_DELAY_SHORT } else { ARG_DELAY_LONG }.len()..].to_string();
                let multiplier: u32 = match delay_str {
                    _ if delay_str.ends_with("s") => { 1 }
                    _ if delay_str.ends_with("m") => { 60 }
                    _ if delay_str.ends_with("h") => { 60*60 }
                    _ => { unreachable!() }
                };
                let delay: u32 = multiplier * delay_str[..delay_str.len()-1].parse::<u32>().unwrap();
                config.delay = Some(delay);
            }
            arg_path if arg.starts_with(ARG_PATH_SHORT) || arg.starts_with(ARG_PATH_LONG) => {
                let mut path_str: String = arg_path[if arg_path.starts_with(ARG_PATH_SHORT) { ARG_PATH_SHORT } else { ARG_PATH_LONG }.len()..].to_string();
                if path_str.starts_with("~") {
                    let user_name: String = Command::new("whoami")
                        .output()
                        .unwrap()
                        .stdout
                        .to_string()
                        .trim()
                        .to_string();
                    path_str = "/home/".to_string() + &user_name + &path_str[1..];
                }
                let path: &Path = Path::new(&path_str);
                assert!(path.is_dir());
                config.wallpapers_path = Some(path.to_path_buf());
            }
            _ => {
                println!("Unkown arg: `{arg}`");
                return;
            }
        }
    }

    println!("config = {config:#?}");

    config.init_wallpapers();

    loop {
        // TODO: impl time dependent wallpaper chooser
        let random_wallpaper: &Wallpaper = &config.wallpapers.choose(&mut rand::thread_rng()).unwrap();
        let path_str: String = random_wallpaper.path.display().to_string();
        println!("setting wallpaper: {path_str}");
        set_wallpaper(&random_wallpaper.path).unwrap();
        println!("Slepping...");
        thread::sleep(time::Duration::from_secs(config.delay.unwrap() as u64));
    }
}

