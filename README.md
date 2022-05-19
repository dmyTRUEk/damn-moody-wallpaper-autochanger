# Damn Moody Wallpaper Autochanger

## Installation:
- From source:
  ```
  git clone https://github.com/dmyTRUEk/damn-moody-wallpaper-autochanger
  cd damn-moody-wallpaper-autochanger
  cargo build --release
  cp target/release/damn-moody-wallpaper-autochanger ~/.local/bin/
  ```

## Configuration:
- Delay: `-d` or `--delay`, examples: `-d=30s`, `-d=5m`, `--delay=2h`.
- Wallpaper path: `-p` or `--path=`, example: `-p=~/Pictures/Wallpapers/`.

## Interaction:
- Skip this wallpaper by sending `SIGINT` signal to program, like this:
  ```
  pkill -SIGINT damn-moody-wall
  ```
  so you can set any keybinding to skip wallpaper using your DE/WM configuration.

