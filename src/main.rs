use std::borrow::BorrowMut;

use bevy::{
	core::FrameCount, input::keyboard, math::{bounding::*, primitives::*, vec3}, prelude::*, render::camera::ScalingMode, sprite::MaterialMesh2dBundle, time::Stopwatch, transform::commands, window::{PresentMode, WindowResolution}
};

use rand::seq::IteratorRandom;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

mod config;
use config::consts::*;

mod debug;

// ECS
// E - Entity
// C - Component
// S - System
// Entities: Game, Checkboard, Snake, Przysmak
// Components: 
//		- Score(Game)
//		- tail_length(Snake)
//		- size(Checkboard)
// Systems:
//		- Draw(Game, Checkboard, Snake, Przysmak)
// 		- {Start, End}(Game)
//		- Move(Snake)
//		- Spawn(Przysmak)

fn main() {
	App::new()
		.init_resource::<debug::WorldCoords>()
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                position: WindowPosition::Centered(MonitorSelection::Current),
				title: String::from(WINDOW_TITLE),
				resizable: false,
                visible: false,
				present_mode: PresentMode::AutoVsync,
				..default()
			}),
			..default()
		}))
		.add_systems(Startup, (
			setup,
			spawn_snake,
		))
		.add_systems(FixedUpdate, (
			snake_movement_input,
			snake_movement,
			game_over,
			check_for_collisions,
			snake_growth,
			spawn_random_food,
			make_snake_visible,
		).chain())
		.add_systems(Update, (
			make_visible,
			tile_color_change,
			bevy::window::close_on_esc,
		))
		.insert_resource(Time::<Fixed>::from_seconds(0.15))
		.insert_resource(SnakeSegments::default())
		.insert_resource(LastTailPosition::default())
		.add_event::<GrowthEvent>()
		.add_event::<GameOverEvent>()
		.run();
}

#[derive(Component)]
struct Wall;

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	commands.spawn((Camera2dBundle {
		projection: OrthographicProjection {
			scaling_mode: ScalingMode::AutoMax { max_width: WINDOW_WIDTH, max_height: WINDOW_HEIGHT },
			..default()
		},
		transform: Transform::from_xyz(0., 0., 5.),
		..default()
	}, MainCamera));

	let mut vec_xyz = STARTER_VEC;
	for i in 1..=SQUARE_AMOUNT as i32 {
		for j in 1..=SQUARE_AMOUNT as i32 {
			let is_even: bool = (i + j) % 2 == 0;
			let [r, g, b, _a] = if is_even {
				TILE1_COLOR
			} else {
				TILE2_COLOR
			};

			commands.spawn((MaterialMesh2dBundle {
				mesh: meshes.add(Rectangle::default()).into(),
				transform: Transform {
					translation: vec_xyz,
					scale: Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 0.),
					..default()
				},
				material: materials.add(ColorMaterial::from(Color::rgb_u8(r, g, b))),
				..default()
			},
			Tile,
		));

			vec_xyz.x += SQUARE_SIZE;
			println!("vec_xyz = {vec_xyz}");
		}
		vec_xyz.x = STARTER_VEC.x;
		vec_xyz.y += SQUARE_SIZE; 
	}

	for location in WallLocation::iter() {
		commands.spawn(WallBundle::new(&location))
			.with_children(|parent| {
				parent.spawn(CollisionWallBundle::new(&location));
			});
	}
}

fn spawn_random_food(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	tile_query: Query<&Transform, With<Tile>>,
	food_query: Query<&SnakeTreat>,
	segments: Query<Entity, With<SnakeSegment>>,
	positions: Query<&Transform, With<SnakeMarker>>,
) {
	if let Ok(_food) = food_query.get_single() {
		return;
	} else {
		let mut rng = rand::thread_rng();
		let tile_transform: Option<&Transform> = tile_query.iter().choose(&mut rng);
		// This will fail only if query returns no Tiles
		let tile_transform = tile_transform.unwrap();
		let mut food_transform = *tile_transform;
		food_transform.translation.z = 4.;
		food_transform.scale.x *= 0.75;
		food_transform.scale.y *= 0.75;

		let segment_positions = segments
			.iter()
			.map(|e| positions.get(e).unwrap().clone())
			.collect::<Vec<Transform>>();

		for seg_pos in segment_positions {
			let collision = 
			Aabb2d::new(
				seg_pos.translation.truncate(),
				seg_pos.scale.truncate() / 2.
			).intersects(
			&Aabb2d::new(
				food_transform.translation.truncate(),
				food_transform.scale.truncate() / 2.
			));

			if collision {
				return;
			}
		}
		// let collision = 
		// Aabb2d::new(
		// 	snake_head_transform.translation.truncate(),
		// 	snake_size / 2.
		// ).intersects(
		// &Aabb2d::new(
		// 	transform.translation.truncate(),
		// 	transform.scale.truncate() / 2.
		// ));

		commands.spawn((MaterialMesh2dBundle {
			mesh: meshes.add(Circle::default()).into(),
			transform: food_transform,
			material: materials.add(ColorMaterial::from(Color::hex(FOOD_COLOR_HEX).unwrap())),
			..default()
			},
			SnakeTreat,
			Collider,
		));
	}
}

#[derive(Component)]
struct SnakeSegment;

#[derive(Resource, Default, Deref, DerefMut)]
struct SnakeSegments(Vec<Entity>);

fn spawn_snake_segment(
	mut commands: Commands,
	position: Vec2
) -> Entity {
	commands.spawn((SpriteBundle {
		sprite: Sprite {
			color: Color::hex(SNAKE_SEGMENT_COLOR_HEX).unwrap(),
			..default()
		},
		transform: Transform {
			scale: Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 5.),
			translation: Vec2::extend(position, 4.),
			..default()
		},
		..default()
	},
	SnakeSegment,
	SnakeMarker,
	)).id()
}

#[derive(Component)]
struct SnakeTreat;

#[derive(Resource, Default)]
struct LastTailPosition(Option<Vec3>);

#[derive(Component)]
pub struct SnakeMarker;

#[derive(Component)]
struct SnakeHead {
	direction: Direction,
}

fn spawn_snake(
    mut commands: Commands,
	mut segments: ResMut<SnakeSegments>,
) {
	*segments = SnakeSegments(vec![
		commands.spawn((SpriteBundle {
			sprite: Sprite {
				color: Color::hex(SNAKE_HEAD_COLOR_HEX).unwrap(),
				..default()
			},
			transform: Transform {
				scale:  Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 5.),
				translation: STARTER_SNAKE_VEC,
				..default()
			},
			//visibility: Visibility::Hidden,
			..default()
		},
		SnakeMarker,
		SnakeHead { direction: Direction::Right },
		)).id(),
	]);
	let last_tail_position = LastTailPosition(Some(Vec3::new(-20.0, 20.0, 4.0)));
	segments.push(spawn_snake_segment(commands, last_tail_position.0.unwrap().truncate()));
	//spawn_snake_segment(commands, STARTER_SNAKE_VEC.truncate().min(Vec2::new(SQUARE_SIZE, 0.)));
}
//segments.push(spawn_snake_segment(commands, last_tail_position.0.unwrap().truncate()));
// /// detect new enemies and print their health
// fn debug_new_hostiles(
//     query: Query<(Entity, &Health), Added<Enemy>>,
// ) {
//     for (entity, health) in query.iter() {
//         eprintln!("Entity {:?} is now an enemy! HP: {}", entity, health.hp);
//     }
// }

fn make_snake_visible(mut last_time: Local<f32>, time: Res<Time>, fixed_time: Res<Time<Fixed>>, mut query: Query<&mut Visibility, Added<SnakeHead>>) {
	if time.elapsed_seconds() > 0.45 {
		for mut visibility in query.iter_mut() {
			*visibility = Visibility::Visible;
		}
	}
}

fn tile_color_change(
	keyboard_input: Res<ButtonInput<KeyCode>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	mut tiles: Query<&mut Handle<ColorMaterial>, With<Tile>>,
) {
	if keyboard_input.just_pressed(KeyCode::Space) {
		for color in &mut tiles.iter() {
			let mut color_mat = materials.get_mut(color).unwrap();
			
			let current_rgba: [u8; 4] = color_mat.color.as_rgba_u8();
			
			let [r, g, b, _a] = if current_rgba == TILE1_COLOR {
				TILE2_COLOR
			} else {
				TILE1_COLOR
			};

			color_mat.color = Color::rgb_u8(r, g, b);
		}
	}
}

fn snake_movement_input(
	keyboard_input: Res<ButtonInput<KeyCode>>,
	mut head_positions: Query<&mut SnakeHead>,
) {
	if let Some(mut head) = head_positions.iter_mut().next() {
		let dir: Direction =
			if keyboard_input.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
				Direction::Left
			} else if keyboard_input.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
				Direction::Right
			} else if keyboard_input.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
				Direction::Down
			} else if keyboard_input.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
				Direction::Up
			} else {
				head.direction
			};
		
		if dir != head.direction.opposite() {
			head.direction = dir;
		}
	}
}

#[derive(Component)]
struct RenderTime {
	stopwatch: Stopwatch,
}

fn snake_movement(
	segments: ResMut<SnakeSegments>,
	mut heads: Query<(Entity, &SnakeHead)>,
	mut positions: Query<&mut Transform, With<SnakeMarker>>,
	mut last_tail_position: ResMut<LastTailPosition>,
	mut game_over_event_writer: EventWriter<GameOverEvent>,
) {
	if let Some((head_entity, head)) = heads.iter_mut().next() {
		let segment_positions = segments
			.iter()
			.map(|e| positions.get_mut(*e).unwrap().translation)
			.collect::<Vec<Vec3>>();
		let mut head_pos = positions.get_mut(head_entity).unwrap();

		match &head.direction {
			Direction::Left => {
				head_pos.translation.x -= SQUARE_SIZE;
			}
			Direction::Right => {
				head_pos.translation.x += SQUARE_SIZE;
			}
			Direction::Down => {
				head_pos.translation.y -= SQUARE_SIZE;
			}	
			Direction::Up => {
				head_pos.translation.y += SQUARE_SIZE;
			}
		};
		let head_pos = positions.get(head_entity).unwrap().to_owned();

		segment_positions
			.iter()
			.zip(segments.iter().skip(1))
			.for_each(|(pos, segment)| {
				positions.get_mut(*segment).unwrap().translation = *pos;
				if *pos == head_pos.translation {
					game_over_event_writer.send(GameOverEvent);
				}
			});
		*last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));
	}
}

fn snake_growth(
	commands: Commands,
	last_tail_position: Res<LastTailPosition>,
	mut segments: ResMut<SnakeSegments>,
	mut growth_event_reader: EventReader<GrowthEvent>,
) {
	if growth_event_reader.read().next().is_some() {
		segments.push(spawn_snake_segment(commands, last_tail_position.0.unwrap().truncate()));
	}

}

// direction
#[derive(PartialEq, Copy, Clone)]
enum Direction {
	Left,
	Right,
	Up,
	Down
}

impl Direction {
	fn opposite(self) -> Self {
		match self {
			Self::Left => Self::Right,
			Self::Right => Self::Left,
			Self::Up => Self::Down,
			Self::Down => Self::Up,
		}
	}
}

// window
fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    if frames.0 == 3 {
        window.single_mut().visible = true;
    }
}

// collisions
#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Event, Default)]
struct GrowthEvent;

#[derive(Event, Default)]
struct GameOverEvent;

// TODO: Refactor walls
// Check wall location and apply proper bounds
fn check_for_collisions(
	mut commands: Commands,
	mut growth_event_writer: EventWriter<GrowthEvent>,
	mut game_over_event_writer: EventWriter<GameOverEvent>,
	mut snake_head_query: Query<&mut Transform, (With<SnakeHead>, Without<Collider>)>,
	collider_query: Query<(Entity, &Transform, Option<&SnakeTreat>), With<Collider>>,
) {
	let mut snake_head_transform = snake_head_query.single_mut(); 
	let snake_size = snake_head_transform.scale.truncate();

	for (collider_entity, transform, maybe_przysmak) in &collider_query {
		// let collision = collide(
		// 	snake_head_transform.translation,
		// 	snake_size,
		// 	transform.translation,
		// 	transform.scale.truncate(),
		// );

		let collision = 
		Aabb2d::new(
			snake_head_transform.translation.truncate(),
			snake_size / 2.
		).intersects(
		&Aabb2d::new(
			transform.translation.truncate(),
			transform.scale.truncate() / 2.
		));

		if collision {
			println!("Collision at {}", transform.translation);
			
			if maybe_przysmak.is_some() {
				commands.entity(collider_entity).despawn();
				growth_event_writer.send(GrowthEvent);

			} else {
				game_over_event_writer.send(GameOverEvent);
				//(*snake_head_transform).translation = Vec3::new(20., 20., 4.);
				(*snake_head_transform).scale = Vec3::new(0.0, 0.0, 0.0);
			}
			
		}
	}
}

#[derive(EnumIter)]
enum WallLocation {
	Left,
	Right,
	Top,
	Bottom,
}

impl WallLocation {
	// x
	const LEFT_WALL: f32 = -(SQUARE_AMOUNT / 2.) * SQUARE_SIZE;
	const RIGHT_WALL: f32 = (SQUARE_AMOUNT / 2.) * SQUARE_SIZE;
	// y
	const TOP_WALL: f32 = (SQUARE_AMOUNT / 2.) * SQUARE_SIZE;
	const BOTTTOM_WALL: f32 = -(SQUARE_AMOUNT / 2.) * SQUARE_SIZE;

	fn position(&self) -> Vec2 {
		match self {
			WallLocation::Left => Vec2::new(Self::LEFT_WALL, 0.),
			WallLocation::Right => Vec2::new(Self::RIGHT_WALL, 0.),
			WallLocation::Top => Vec2::new(0., Self::TOP_WALL),
			WallLocation::Bottom => Vec2::new(0., Self::BOTTTOM_WALL),
		}
	}

	fn collision_position(&self) -> Vec2 {
		match self {
			WallLocation::Left => Vec2::new(Self::LEFT_WALL - 10., 0.),
			WallLocation::Right => Vec2::new(Self::RIGHT_WALL + 10., 0.),
			WallLocation::Top => Vec2::new(0., Self::TOP_WALL + 10.),
			WallLocation::Bottom => Vec2::new(0., Self::BOTTTOM_WALL - 10.),
		}
	}

	fn size(&self) -> Vec2 {
		match self {
			WallLocation::Left | WallLocation::Right => {
				Vec2::new( 5., SQUARE_AMOUNT * SQUARE_SIZE)
			}
			WallLocation::Top | WallLocation::Bottom => {
				Vec2::new(SQUARE_AMOUNT * SQUARE_SIZE, 5.)
			}
		}
	}
}

#[derive(Bundle)]
struct CollisionWallBundle {
	sprite_bundle: SpriteBundle,
	collider: Collider
}

impl CollisionWallBundle {
	fn new(location: &WallLocation) -> CollisionWallBundle {
		CollisionWallBundle {
			sprite_bundle: SpriteBundle {
				sprite: Sprite {
					..default()
				},
				transform: Transform {
					translation: (*location).collision_position().extend(4.),
					scale: (*location).size().extend(5.),
					..default()
				},
				visibility: Visibility::Hidden,
				..default()
			},
			collider: Collider,
		}
	}
}

#[derive(Bundle)]
struct WallBundle {
	sprite_bundle: SpriteBundle,
}

impl WallBundle {
	fn new(location: &WallLocation) -> WallBundle {
		WallBundle {
			sprite_bundle: SpriteBundle {
				sprite: Sprite {
					color: Color::hex("#000000").unwrap(),
					..default()
				},
				transform: Transform {
					translation: (*location).position().extend(4.5),
					scale: (*location).size().extend(5.),
					..default()
				},
				..default()
			},
		}
	}
}

fn game_over(
	mut commands: Commands,
	mut reader: EventReader<GameOverEvent>,
	segments_res: ResMut<SnakeSegments>,
	food: Query<Entity, With<SnakeTreat>>,
	segments: Query<Entity, With<SnakeSegment>>,
	snake_head: Query<Entity, With<SnakeHead>>,
) {
	if reader.read().next().is_some() {
		commands.entity(snake_head.single()).despawn();
		for ent in food.iter().chain(segments.iter()) {
			commands.entity(ent).despawn();
		}
		spawn_snake(commands, segments_res);
	}
}