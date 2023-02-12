mod api;
mod auth;
mod types;
mod util;

pub use api::VerticalLifeClient;
pub use auth::VerticalLifeAuthClient;
pub use types::*;
pub use util::*;

pub const WALL_CATEGORIES: &[&str] = &["gym_bouldering", "gym_sportclimbing"];
