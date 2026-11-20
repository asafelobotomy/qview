//! qview's rules that need no Qt: which pictures sit next to the open one and in what order,
//! the settings file, the trash, where mark-ups are saved, the command line, and the wallpaper
//! command. The window (crates/qview) does the drawing.

pub mod args;
pub mod config;
pub mod defaults;
pub mod folder;
pub mod markup;
pub mod trash;
pub mod wallpaper;
