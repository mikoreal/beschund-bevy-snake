use crate::MainCamera;
use crate::config::consts::*;
use bevy::{prelude::*, window::PrimaryWindow};

// cursor to pixel

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
pub struct WorldCoords(Vec2);

pub fn my_cursor_system(
    mut mycoords: ResMut<WorldCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
        eprintln!("World coords: {}/{}", world_position.x, world_position.y);
    }
}

pub fn draw_helper_grid(mut commands: Commands) {
	let grid_size = 400;
	let cell_size = SQUARE_SIZE;

	for i in 0..=grid_size {
        let position = i as f32 * cell_size - grid_size as f32 / 2.0 * cell_size;
        println!("{}", position);
        // Vertical line
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(2.0, cell_size * grid_size as f32)),
                color: Color::rgba(0.5, 0.5, 0.5, 0.3),
                ..Default::default()
            },
            transform: Transform::from_xyz(position, 0.0, 2.0),
            ..Default::default()
        });
        
        // Horizontal line
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(cell_size * grid_size as f32, 2.0)),
                color: Color::rgba(0.5, 0.5, 0.5, 0.3),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, position, 2.0),
            ..Default::default()
        });
    }
}