/// damn moody wallpaper autochanger

use std::env::{args, self};
use std::f32::consts::TAU;
use std::path::Path;
use std::process::{Command, exit as sys_exit};
use std::thread;
use std::time;

use chrono::{NaiveTime, Timelike};
use image::GenericImageView;
use rand::{Rng, thread_rng};
use walkdir::WalkDir;



#[derive(Clone, Copy)]
enum DEWM {
    // Desktop Environments
    Budgie,
    Cinnamon,
    Deepin,
    Gnome,
    Kde,
    Lxde,
    Lxqt,
    Mate,
    Xfce,
    // Window Managers
    Awesome,
    Bspwm,
    Dwm,
    I3,
    Qtile,
    Sway,
    Wayfire,
    Xmonad,
}



fn get_de_wm() -> DEWM {
    let xdg_current_desktop: String = match env::var("XDG_CURRENT_DESKTOP") {
        Ok(val) => { val }
        Err(_e) => { panic!("environment variable XDG_CURRENT_DESKTOP not found") }
    };
    // println!("xdg_current_desktop = {xdg_current_desktop}");
    match xdg_current_desktop {
        _ if xdg_current_desktop.ends_with("GNOME") => {
            DEWM::Gnome
        }
        _ if xdg_current_desktop.ends_with("SWAY") => {
            todo!("Check is this is correct way to check if this is sway");
            DEWM::Sway
        }
        _ => { todo!() }
    }
}



fn set_wallpaper(dewm: DEWM, path_str: &str) {
    assert!(path_str.len() > 0);
    if Path::new(path_str).is_file() == false {
        println!("File unavailable: {path_str}");
        // exit("File unavailable");
    }
    match dewm {
        DEWM::Gnome => {
            let res = Command::new("gsettings")
                .env("GSETTINGS_BACKEND", "dconf")
                .arg("set")
                .arg("org.gnome.desktop.background")
                .arg("picture-uri")
                .arg(format!("file://{path_str}"))
                .output();
            if res.is_err() {
                exit("Error while executing command to set GNOME light wallpaper");
            }
            let res = Command::new("gsettings")
                .env("GSETTINGS_BACKEND", "dconf")
                .arg("set")
                .arg("org.gnome.desktop.background")
                .arg("picture-uri-dark")
                .arg(format!("file://{path_str}"))
                .output();
            if res.is_err() {
                exit("Error while executing command to set GNOME dark wallpaper");
            }
        }
        DEWM::Sway => {
            todo!()
        }
        _ => { todo!() }
    }
}



pub trait ExtensionVecU8ToString {
    fn to_string(self) -> String;
}
impl ExtensionVecU8ToString for Vec<u8> {
    fn to_string(self) -> String {
        String::from_utf8(self).unwrap()
    }
}



/// random number from gauss distribution
fn random_gauss(mu: f32, sigma: f32) -> f32 {
    // from python's `random.gauss`:
    // When x and y are two variables from [0, 1), uniformly distributed, then
    // cos(2*pi*x)*sqrt(-2*ln(1-y))
    // sin(2*pi*x)*sqrt(-2*ln(1-y))
    // are two *independent* variables with normal distribution (mu=0, sigma=1).
    let mut rng = thread_rng();
    let x: f32 = rng.gen_range(0.0..1.0);
    let y: f32 = rng.gen_range(0.0..1.0);
    let t: f32 = (TAU*x).cos() * (-2.0*(1.0-y).ln()).sqrt();
    mu + t * sigma
}



fn time_to_desired_brightness(time: NaiveTime) -> f32 {
    let hour = time.hour();
    match hour {
        _ if (5 <= hour && hour < 21) => { // day
            0.7
        }
        _ if (20 <= hour && hour < 21) => { // evening
            0.2
        }
        _ if (21 <= hour && hour < 22) => { // night
            0.1
        }
        _ if (22 <= hour && hour < 24) || (0 <= hour && hour < 6) => { // deep night
            0.05
        }
        _ => { unreachable!() }
    }
}



fn generate_brightness_by_gauss(desired_brightness: f32) -> f32 {
    let mut random_brightness: Option<f32> = None;
    while random_brightness.is_none() {
        let rb: f32 = random_gauss(desired_brightness, desired_brightness/2.0);
        if 0.0 <= rb && rb <= 1.0 {
            random_brightness = Some(rb);
        }
    }
    random_brightness.unwrap()
}

fn smart_choose(wallpapers: &Vec<Wallpaper>) -> &Wallpaper {
    assert!(wallpapers.len() > 0);
    let desired_brightness: f32 = time_to_desired_brightness(chrono::Local::now().time());
    println!("desired_brightness: {desired_brightness}");

    let random_brightness: f32 = generate_brightness_by_gauss(desired_brightness);
    println!("random_brightness: {random_brightness}");

    let mut closest_i: usize = 0;
    for i in 0..wallpapers.len() {
        if (wallpapers[i].brightness-random_brightness).abs() < (wallpapers[closest_i].brightness-random_brightness).abs() {
            closest_i = i;
        }
    }
    &wallpapers[closest_i]
}



fn calc_image_brightness(path_str: &str) -> Option<f32> {
    let image = image::open(path_str);
    if image.is_err() { return None; }
    let image = image.unwrap();
    let mut brightness: u64 = 0;
    for (_w, _h, pixel) in image.pixels() {
        brightness += pixel.0[0] as u64; // red
        brightness += pixel.0[1] as u64; // green
        brightness += pixel.0[2] as u64; // blue
        // brightness += pixel.0[3] as u64; // alpha
    }
    Some((brightness as f64 / (4.0 * 255.0 * image.dimensions().0 as f64 * image.dimensions().1 as f64)) as f32)
}



#[derive(Clone, Debug)]
struct Wallpaper {
    path_str: String,
    brightness: f32,
}



#[derive(Debug)]
struct Config {
    delay: Option<u32>,
    wallpapers: Vec<Wallpaper>,
}
impl Config {
    fn new() -> Self {
        Config {
            delay: None,
            wallpapers: vec![],
        }
    }

    fn load_wallpapers(&mut self, path_str: &str) {
        for entry in WalkDir::new(path_str) {
            let path: &Path = entry.as_ref().unwrap().path();
            if path.is_dir() { continue; }
            let path_str: String = path.display().to_string();
            let brightness: Option<f32> = calc_image_brightness(&path_str);
            if brightness.is_none() {
                let path_str: String = path_str;
                println!("Skipping {path_str}");
                continue;
            }
            let brightness: f32 = brightness.unwrap();
            println!("{}", path_str);
            println!("brightness = {brightness}");
            let mut wallpaper: Wallpaper = Wallpaper { path_str, brightness };
            wallpaper.path_str.shrink_to_fit();
            self.wallpapers.push(wallpaper);
        }
    }
}



fn exit(msg: &str) {
    println!("{msg}");
    sys_exit(1);
}


fn generate_config_from_args() -> Config {
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
                if Path::new(&path_str).is_dir() == false {
                    exit("Provided path is not a valid dir.");
                }
                println!("Loading wallpapers from {path_str}");
                config.load_wallpapers(&path_str);
            }
            _ => {
                exit(&format!("Unkown arg: `{arg}`"));
            }
        }
    }
    config.wallpapers.shrink_to_fit();
    config
}



fn main() {
    let config: Config = generate_config_from_args();
    println!("config = {config:#?}\n");

    let dewm: DEWM = get_de_wm();

    loop {
        println!();
        let random_wallpaper: &Wallpaper = smart_choose(&config.wallpapers);
        println!("Setting wallpaper: {path_str}", path_str=random_wallpaper.path_str);
        set_wallpaper(dewm, &random_wallpaper.path_str);
        println!("Sleeping {d}s...", d=config.delay.unwrap());
        thread::sleep(time::Duration::from_secs(config.delay.unwrap() as u64));
    }
}

