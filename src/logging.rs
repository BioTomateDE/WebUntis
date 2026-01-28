use std::io::Write;

use chrono::Utc;
use colored::{Color, Colorize as _};
use env_logger::{Builder, Env};
use log::Level;

pub fn init() {
    let mut builder = Builder::new();

    builder.parse_env(get_env());

    builder.format(|f, record| {
        let time = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string().dimmed();
        let color = color_by_level(record.level());
        let level = level_to_str(record.level()).color(color);
        let target = record.target().dimmed();
        let message = record.args().to_string().color(color);

        writeln!(f, "{time} [{level}@{target}] {message}")
    });

    builder.init();
}

fn get_env() -> Env<'static> {
    let default_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    Env::default().default_filter_or(default_level)
}

fn color_by_level(level: Level) -> Color {
    match level {
        Level::Trace => Color::Magenta,
        Level::Debug => Color::Blue,
        Level::Info => Color::Green,
        Level::Warn => Color::Yellow,
        Level::Error => Color::Red,
    }
}

fn level_to_str(level: Level) -> &'static str {
    match level {
        Level::Trace => "T",
        Level::Debug => "D",
        Level::Info => "I",
        Level::Warn => "W",
        Level::Error => "E",
    }
}
