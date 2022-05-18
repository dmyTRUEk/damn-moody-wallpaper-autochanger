use std::{path::{Path, PathBuf}, process::{Command, Output}, io::Error, env::args};


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
    let xdg_current_desktop: String = Command::new("echo")
        .arg("$XDG_CURRENT_DESKTOP")
        .output()
        .unwrap()
        .stdout
        .to_string();
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
    match get_de_wm() {
        DEWM::Gnome => {
            Command::new("gsettings")
                .env("GSETTINGS_BACKEND", "dconf")
                .arg("set")
                .arg("org.gnome.desktop.background")
                .arg("picture-uri")
                .arg("file://{path}")
                .output()?;
            Command::new("gsettings")
                .env("GSETTINGS_BACKEND", "dconf")
                .arg("set")
                .arg("org.gnome.desktop.background")
                .arg("picture-uri-dark")
                .arg("file://{path}")
                .output()
        }
        DEWM::Sway => {
            todo!()
        }
    }
}

#[derive(Debug)]
struct Config {
    delay: Option<u32>,
    wallpapers_path: Option<PathBuf>,
}
impl Config {
    fn new() -> Self {
        Config {
            delay: None,
            wallpapers_path: None,
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
                todo!("*s, *m, *h");
                let delay: u32 = delay_str.parse().unwrap();
                config.delay = Some(delay);
            }
            arg_path if arg.starts_with(ARG_PATH_SHORT) || arg.starts_with(ARG_PATH_LONG) => {
                let path_str: String = arg_path[if arg_path.starts_with(ARG_PATH_SHORT) { ARG_PATH_SHORT } else { ARG_PATH_LONG }.len()..].to_string();
                let path: &Path = Path::new(&path_str);
                config.wallpapers_path = Some(path.to_path_buf());
            }
            _ => {
                println!("Unkown arg: `{arg}`");
                return;
            }
        }
    }

    println!("config = {config:#?}");
}

