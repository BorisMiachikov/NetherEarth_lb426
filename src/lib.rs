mod ai;
mod app;
mod audio;
mod camera;
mod combat;
mod command;
mod core;
#[cfg(feature = "debug_tools")]
mod debug;
mod economy;
mod editor;
pub mod localization;
mod map;
mod movement;
mod player;
mod robot;
mod save;
mod structure;
mod ui;

pub use app::plugin::AppPlugin;
