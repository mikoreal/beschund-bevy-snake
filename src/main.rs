use bevy::{
	core::FrameCount, input::keyboard, prelude::*, render::camera::ScalingMode, sprite::{
		collide_aabb::{collide, Collision}, MaterialMesh2dBundle
	}, transform::commands, window::{PresentMode, WindowResolution}
};

use rand::seq::IteratorRandom;

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
			//debug::draw_helper_grid
		))
		//.add_systems(Update, (make_visible, snake_movement, tile_color_change, debug::my_cursor_system))
		.add_systems(FixedUpdate, (
			snake_movement_input,
			snake_movement,
			tile_color_change,
			//debug::my_cursor_system,
			check_for_collisions,
			snake_growth,
			spawn_random_food
		).chain())
		.add_systems(Update, make_visible)
		.insert_resource(Time::<Fixed>::from_seconds(0.15))
		.insert_resource(SnakeSegments::default())
		.insert_resource(LastTailPosition::default())
		.add_event::<GrowthEvent>()
		.run();
}

#[derive(Component)]
struct Wall {
	visible: bool
}

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
	for i in 1..=SQUARE_AMOUNT as i32 { // for j in 1..17
		for j in 1..=SQUARE_AMOUNT as i32 {
			let is_even: bool = (i + j) % 2 == 0;
			let [r, g, b, _a] = if is_even {
				TILE1_COLOR
			} else {
				TILE2_COLOR
			};

			commands.spawn((MaterialMesh2dBundle {
				mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
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

	commands.spawn(WallBundle::new(WallLocation::Left));
	commands.spawn(WallBundle::new(WallLocation::Right));
	commands.spawn(WallBundle::new(WallLocation::Bottom));
	commands.spawn(WallBundle::new(WallLocation::Top));

	commands.spawn(WallBundle::new(WallLocation::LeftHide));
	commands.spawn(WallBundle::new(WallLocation::RightHide));
	commands.spawn(WallBundle::new(WallLocation::BottomHide));
	commands.spawn(WallBundle::new(WallLocation::TopHide));
}

fn spawn_random_food(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	tile_query: Query<&Transform, With<Tile>>,
	food_query: Query<&Przysmak>
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

		commands.spawn((MaterialMesh2dBundle {
			mesh: meshes.add(Mesh::from(shape::Circle::default())).into(),
			transform: food_transform,
			material: materials.add(ColorMaterial::from(Color::hex(FOOD_COLOR_HEX).unwrap())),
			..default()
			},
			Przysmak,
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
	SnakeMarker,)).id()

}

/*
fn spawn_snake(
    mut commands: Commands
) {
    commands.spawn((SpriteBundle {
        sprite: Sprite {
            color: Color::hex(SNAKE_HEAD_COLOR_HEX).unwrap(),
            ..default()
        },
        transform: Transform {
            scale:  Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 5.),
            translation: Vec3::new(20., 20., 4.),
            ..default()
        },
		..default()
    },
	SnakeMarker,
	SnakeHead,
	));
}
 */

#[derive(Component)]
struct Przysmak;

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
	mut segments: ResMut<SnakeSegments>
) {
	*segments = SnakeSegments(vec![
		commands.spawn((SpriteBundle {
			sprite: Sprite {
				color: Color::hex(SNAKE_HEAD_COLOR_HEX).unwrap(),
				..default()
			},
			transform: Transform {
				scale:  Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 5.),
				translation: Vec3::new(20., 20., 4.),
				..default()
			},
			..default()
		},
		SnakeMarker,
		SnakeHead { direction: Direction::Right },
		)).id(),
		spawn_snake_segment(commands, Vec2::new(-20., 20.)),
	]);
}

fn tile_color_change(
	keyboard_input: Res<Input<KeyCode>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	mut tiles: Query<&mut Handle<ColorMaterial>, With<Tile>>,
) {
	if keyboard_input.just_pressed(KeyCode::Space) {
		for color in &mut tiles.iter() {
			let mut color_mat = materials.get_mut(color).unwrap();
			
			let current_rgba: [u8; 4] = color_mat.color.as_rgba_u8();
			//ColorMaterial::from(Color::rgb_u8(r, g, b))
			let [r, g, b, a] = if current_rgba == TILE1_COLOR {
				TILE2_COLOR
			} else {
				TILE1_COLOR
			};

			color_mat.color = Color::rgb_u8(r, g, b);
		}
	}
}

fn snake_movement_input(
	keyboard_input: Res<Input<KeyCode>>,
	mut head_positions: Query<&mut SnakeHead>,
) {
	if let Some(mut head) = head_positions.iter_mut().next() {
		let dir: Direction =
			if keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
				Direction::Left
			} else if keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) {
				Direction::Right
			} else if keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) {
				Direction::Down
			} else if keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) {
				Direction::Up
			} else {
				head.direction
			};
		
		if dir != head.direction.opposite() {
			head.direction = dir;
		}
	}
}

fn snake_movement(
	segments: ResMut<SnakeSegments>,
	mut heads: Query<(Entity, &SnakeHead)>,
	mut positions: Query<&mut Transform, With<SnakeMarker>>,
	mut last_tail_position: ResMut<LastTailPosition>,
) {
	if let Some((head_entity, head)) = heads.iter_mut().next() {
		let segment_positions = segments
			.iter()
			.map(|e| positions.get_mut(*e).unwrap().translation)
			.collect::<Vec<Vec3>>();
		let mut head_pos = positions.get_mut(head_entity).unwrap();

		let mut pos_offset_x: f32 = 0.;
		let mut pos_offset_y: f32 = 0.; 
		match &head.direction {
			Direction::Left => {
				head_pos.translation.x -= 40.;
				//pos_offset_x += SQUARE_SIZE - 5.;
			}
			Direction::Right => {
				head_pos.translation.x += 40.;
				//pos_offset_x += -SQUARE_SIZE + 5.;
			}
			Direction::Down => {
				head_pos.translation.y -= 40.;
				//pos_offset_y += SQUARE_SIZE - 5.;
			}	
			Direction::Up => {
				head_pos.translation.y += 40.;
				//pos_offset_y += -SQUARE_SIZE + 5.;
			}
		};
		segment_positions
			.iter()
			.zip(segments.iter().skip(1))
			.for_each(|(pos, segment)| {
				positions.get_mut(*segment).unwrap().translation = *pos;
				positions.get_mut(*segment).unwrap().translation.x += pos_offset_x;
				positions.get_mut(*segment).unwrap().translation.y += pos_offset_y;
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

fn check_for_collisions(
	mut commands: Commands,
	mut growth_event_writer: EventWriter<GrowthEvent>,
	mut snake_head_query: Query<&mut Transform, (With<SnakeHead>, Without<Collider>)>,
	collider_query: Query<(Entity, &Transform, Option<&Przysmak>, Option<&Wall>), With<Collider>>,
) {
	let mut snake_head_transform = snake_head_query.single_mut(); 
	let snake_size = snake_head_transform.scale.truncate();

	for (collider_entity, transform, maybe_przysmak, maybe_wall) in &collider_query {
		let collision = collide(
			snake_head_transform.translation,
			snake_size,
			transform.translation,
			transform.scale.truncate(),
		);

		if let Some(collision) = collision {
			println!("Collision at {}", transform.translation);
			
			if maybe_przysmak.is_some() {
				commands.entity(collider_entity).despawn();
				growth_event_writer.send(GrowthEvent);

			} else if maybe_wall.is_some() && !maybe_wall.unwrap().visible {
				(*snake_head_transform).translation = Vec3::new(20., 20., 4.);
			}
			
		}
	}
}

enum WallLocation {
	Left,
	Right,
	Top,
	Bottom,
	LeftHide,
	RightHide,
	TopHide,
	BottomHide,
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

			WallLocation::LeftHide => Vec2::new(Self::LEFT_WALL - 10., 0.),
			WallLocation::RightHide => Vec2::new(Self::RIGHT_WALL + 10., 0.),
			WallLocation::TopHide => Vec2::new(0., Self::TOP_WALL + 10.),
			WallLocation::BottomHide => Vec2::new(0., Self::BOTTTOM_WALL - 10.),
		}
	}

	fn size(&self) -> Vec2 {
		match self {
			WallLocation::Left | WallLocation::Right | WallLocation::LeftHide | WallLocation::RightHide => {
				Vec2::new( 5., SQUARE_AMOUNT * SQUARE_SIZE)
			}
			WallLocation::Top | WallLocation::Bottom | WallLocation::TopHide | WallLocation::BottomHide => {
				Vec2::new(SQUARE_AMOUNT * SQUARE_SIZE, 5.)
			}
		}
	}
}

#[derive(Bundle)]
struct WallBundle {
	sprite_bundle: SpriteBundle,
	collider: Collider,
	wall_marker_component: Wall,
}

impl WallBundle {
	fn new(location: WallLocation) -> WallBundle {
		use WallLocation::*;

		let visible = match location {
			LeftHide | RightHide | TopHide | BottomHide => Visibility::Hidden,
			_ => Visibility::Visible,
		};

		WallBundle {
			sprite_bundle: SpriteBundle {
				sprite: Sprite {
					color: Color::hex("#000000").unwrap(),
					..default()
				},
				visibility: visible,
				transform: Transform {
					translation: location.position().extend(3.),
					scale: location.size().extend(5.),
					..default()
				},
				..default()
			},
			collider: Collider,
			wall_marker_component: Wall{ visible: visible == Visibility::Visible },
		}
	}
}
