use bevy::prelude::Resource;

#[derive(Debug)]
pub enum TileRotation {
    Zero = 0,
    Quarter = 1,
    Half = 2,
    ThreeQuarters = 3,
}

#[derive(Debug)]
pub struct Tile {
    pub rules: Vec<String>,
    pub rotation: TileRotation,
    pub file: String,
}

#[derive(Resource, Default)]
pub struct WFCResource {
    pub tiles: Vec<Tile>,
    pub slots: Vec<Vec<usize>>,
    pub grid_width: usize,
}

impl WFCResource {
    pub fn new(grid_width: usize) -> Self {
        WFCResource {
            grid_width,
            ..Default::default()
        }
    }
}
