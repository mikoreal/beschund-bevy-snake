use bevy::prelude::*;

pub const SQUARE_SIZE: f32 = 40.;
pub const SQUARE_AMOUNT: f32 = 16.;

pub const WINDOW_TITLE: &str = "beschund";

pub const WINDOW_WIDTH: f32 = 800.;
pub const WINDOW_HEIGHT: f32 = 800.;

#[derive(Component)]
pub struct Tile;

pub const TILE1_COLOR: [u8; 4] = [48, 61, 78, u8::MAX];
pub const TILE2_COLOR: [u8; 4] = [39, 49, 63, u8::MAX];

pub const STARTER_VEC: Vec3 = Vec3::new(
    -300.,
    -300.,
    3.
);

pub const STARTER_SNAKE_VEC: Vec3 = Vec3::new(20., 20., 4.);

pub const SNAKE_SEGMENT_COLOR_HEX: &str = "#FFFFFF";
pub const SNAKE_HEAD_COLOR_HEX: &str = "#2ECC71";
pub const FOOD_COLOR_HEX: &str = "#30D5C8"; 

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;