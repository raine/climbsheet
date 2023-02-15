use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GymSector {
    pub id: u32,
    pub gym_id: u32,
    pub name: String,
    pub category: String,
    pub cover: Option<String>,
    pub overview: String,
    pub route_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct GymSectorFull {
    pub id: u32,
    pub gym_id: u32,
    pub name: String,
    pub category: String,
    pub cover: Option<String>,
    pub overview: String,
    pub route_count: u32,
    pub walls: Vec<Wall>,
}

#[derive(Debug, Deserialize)]
pub struct Wall {
    pub id: u32,
    pub gym_sector_id: u32,
    pub height: u32,
    pub name: String,
    /// One of "gym_bouldering", "gym_sportclimbing"
    pub category: String,
    pub gym_boulders: Option<Vec<Climb>>,
    pub gym_routes: Option<Vec<Climb>>,
}

impl Wall {
    pub fn climbs(&self) -> impl Iterator<Item = &Climb> {
        let boulders = self.gym_boulders.iter().flatten();
        let routes = self.gym_routes.iter().flatten();
        boulders.chain(routes)
    }
}

#[derive(Debug, Deserialize)]
pub struct Gym {
    pub id: u32,
    pub name: String,
    pub boulder_count: u32,
    pub route_count: u32,
    pub gym_sectors: Vec<GymSector>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Climb {
    pub id: u32,
    pub difficulty: String,
    pub set_at: DateTime<Utc>,
    #[serde(rename = "color_1")]
    pub color: String,
    pub sector_name: String,
    pub parent_name: String,
    pub route_card_label: String,
    pub route_setter: String,
    /// One of "gym_boulder" or "gym_route"
    pub item_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZlagsResponse {
    pub gym_boulders: Vec<Climb>,
    pub gym_routes: Vec<Climb>,
}
