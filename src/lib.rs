mod ai;
mod app;
mod audio;
pub mod spatial;
mod camera;
mod combat;
mod command;
mod core;
#[cfg(feature = "debug_tools")]
mod debug;
#[cfg(feature = "dev")]
mod dev_tools;
mod economy;
mod editor;
pub mod localization;
mod map;
mod movement;
mod player;
mod robot;
pub mod save;
mod structure;
mod ui;

pub use app::plugin::AppPlugin;
