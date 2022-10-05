use colors_transform::Rgb;
use std::{collections::HashMap, path::PathBuf, str::FromStr};

pub struct Config {
    pub fg_normal: tui::style::Color,
    pub fg_highlight: tui::style::Color,
    pub bg_highlight: tui::style::Color,
    pub fg_column_show: tui::style::Color,
    pub bg_column_show: tui::style::Color,
    pub fg_column_hide: tui::style::Color,
    pub bg_column_hide: tui::style::Color,
    pub torrent_search_dir: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Config {
        let xdg = xdg::BaseDirectories::with_prefix("spoiler").unwrap();
        let config_path = xdg.get_config_file("config.toml");

        let config_build = config::Config::builder()
            .add_source(config::File::from(config_path))
            .build();

        let torrent_search_dir = dirs::download_dir();

        let mut config = Config {
            fg_normal: tui::style::Color::Reset,
            fg_highlight: get_rgb("#000".to_string()),
            bg_highlight: get_rgb("#f00".to_string()),
            fg_column_show: get_rgb("#000".to_string()),
            bg_column_show: get_rgb("#0f0".to_string()),
            fg_column_hide: get_rgb("#000".to_string()),
            bg_column_hide: get_rgb("#00f".to_string()),
            torrent_search_dir,
        };

        if let Ok(conf) = config_build {
            for field in conf.try_deserialize::<HashMap<String, String>>().unwrap() {
                let rgb = get_rgb(field.1.to_string());
                match field.0.as_str() {
                    "fg_normal" => config.fg_normal = rgb,
                    "fg_highlight" => config.fg_highlight = rgb,
                    "bg_highlight" => config.bg_highlight = rgb,
                    "fg_column_show" => config.fg_column_show = rgb,
                    "bg_column_show" => config.bg_column_show = rgb,
                    "fg_column_hide" => config.fg_column_hide = rgb,
                    "bg_column_hide" => config.bg_column_hide = rgb,
                    "torrent_search_dir" => {
                        config.torrent_search_dir = PathBuf::from_str(field.1.as_str()).ok()
                    }
                    _ => (),
                }
            }
        }

        config
    }

    pub fn get_style(&self) -> tui::style::Style {
        tui::style::Style::default().fg(self.fg_normal)
    }

    pub fn get_highlight_style(&self) -> tui::style::Style {
        tui::style::Style::default()
            .fg(self.fg_highlight)
            .bg(self.bg_highlight)
    }
}

pub fn get_rgb(color: String) -> tui::style::Color {
    if let Ok(rgb) = Rgb::from_hex_str(&color) {
        tui::style::Color::Rgb(
            colors_transform::Color::get_red(&rgb) as u8,
            colors_transform::Color::get_green(&rgb) as u8,
            colors_transform::Color::get_blue(&rgb) as u8,
        )
    } else {
        tui::style::Color::Reset
    }
}
