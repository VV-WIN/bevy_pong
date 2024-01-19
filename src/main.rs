use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision}, // TODO: Replace with Rapier 2D Physics
    sprite::MaterialMesh2dBundle,
};
// use bevy_rapier2d::prelude::*;
// With the current sprite collide_aabb there's an issue where the velocity of the ball exceeds the speed of the collision detection.
// This causes the ball to pass through the paddle.
// Rapier 2D Physics has a much better collision detection system. We'll use that instead. 
mod menu;

const BALL_WIDTH: f32 = 10.;
const BALL_SPEED: f32 = 5.; 
const PADDLE_SPEED: f32 = 1.;
const PADDLE_WIDTH: f32 = 10.;
const PADDLE_HEIGHT: f32 = 50.;
const GUTTER_HEIGHT: f32 = 20.;

#[derive(Default, States, Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    MainMenu,
    // SettingsMenu,
    // Playing,
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Gutter;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Ai;

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Shape(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Bundle)]
struct BallBundle {
    ball: Ball,
    shape: Shape,
    velocity: Velocity,
    position: Position
}

impl BallBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            ball: Ball,
            shape: Shape(Vec2::new(BALL_WIDTH, BALL_WIDTH)),
            velocity: Velocity(Vec2::new(x, y)),
            position: Position(Vec2::new(0., 0.))
        }
    }
}

#[derive(Bundle)]
struct PaddleBundle {
    paddle: Paddle,
    shape: Shape,
    velocity: Velocity,
    position: Position
}

impl PaddleBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            paddle: Paddle,
            shape: Shape(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
            velocity: Velocity(Vec2::new(0., 0.)),
            position: Position(Vec2::new(x, y))
        }
    }
}

#[derive(Bundle)]
struct GutterBundle {
    gutter: Gutter,
    shape: Shape,
    position: Position,
}

impl GutterBundle {
    fn new(x: f32, y: f32, w: f32) -> Self {
        Self {
            gutter: Gutter,
            shape: Shape(Vec2::new(w, GUTTER_HEIGHT)),
            position: Position(Vec2::new(x, y)),
        }
    }
}
 
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (
            spawn_ball, 
            spawn_paddles,
            spawn_gutters,
            spawn_camera, 
        ))
        .add_systems(Update, (
            move_ball,
            handle_player_input,
            move_paddles.after(handle_player_input),
            project_positions.after(move_ball),
            handle_collisions.after(move_ball),
        ))
        .add_state::<GameState>()
        .run();
}

fn handle_collisions(
    mut ball: Query<(&mut Velocity, &Position, &Shape), With<Ball>>,
    // We can collide with anything else that has a shape and position that is
    // not itself a ball
    other_things: Query<(&Position, &Shape), Without<Ball>>,
) {
    if let Ok((mut ball_velocity, ball_position, ball_shape)) = ball.get_single_mut() {
        for (position, shape) in &other_things {
            if let Some(collision) = collide(
                ball_position.0.extend(0.), // position_a (Vec3)
                ball_shape.0,               // size_a (Vec2)
                position.0.extend(0.),      // position_b (Vec3)
                shape.0,                    // size_b (Vec2)
            ) {
                match collision {
                    Collision::Left => {
                        ball_velocity.0.x *= -1.;
                    }
                    Collision::Right => {
                        ball_velocity.0.x *= -1.;
                    }
                    Collision::Top => {
                        ball_velocity.0.y *= -1.;
                    }
                    Collision::Bottom => {
                        ball_velocity.0.y *= -1.;
                    }
                    Collision::Inside => {
                        // Do nothing
                    }
                }
            }
        }
    }
}

fn handle_player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut paddle: Query<(&mut Velocity, &Paddle), With<Player>>,
) {
    if let Ok((mut velocity, _)) = paddle.get_single_mut() {
        if keyboard_input.pressed(KeyCode::Up) {
            velocity.0.y = PADDLE_SPEED;
        } else if keyboard_input.pressed(KeyCode::Down) {
            velocity.0.y = -PADDLE_SPEED;
        } else {
            velocity.0.y = 0.;
        }
    }  
} 

fn move_ball(mut ball: Query<(&mut Position, &Velocity), With<Ball>>) {
    if let Ok((mut position, velocity)) = ball.get_single_mut() {
        position.0 += velocity.0 * BALL_SPEED;
    }
}

fn move_paddles(
    mut paddle: Query<(&mut Position, &Velocity), With<Paddle>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();

        for (mut position, velocity) in &mut paddle {
            let new_position = position.0 + velocity.0 * PADDLE_SPEED;
            if new_position.y.abs() < window_height / 2. - GUTTER_HEIGHT - PADDLE_HEIGHT / 2. {
                position.0 = new_position;
            }
        }
    }
}

fn project_positions(mut ball: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut ball {
        transform.translation = position.0.extend(0.);
    }
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("Spawning ball...");

    let mesh = Mesh::from(shape::Circle::new(BALL_WIDTH / 2.0));
    let material = ColorMaterial::from(Color::rgb(1., 0., 0.));

    // Now our mesh shape is derived from the `Shape` we made as a new component
    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(material);

    commands.spawn((
        BallBundle::new(1., 0.),
        MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            material: material_handle,
            ..default()
        },
    ));
}

fn spawn_paddles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    println!("Spawning paddles...");

    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        // right and left of the screen with a bit of padding
        let padding = 50.;
        let right_paddle_x = window_width / 2. - padding;
        let left_paddle_x = -window_width / 2. + padding;

        let mesh = Mesh::from(shape::Quad::new(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)));
        let mesh_handle = meshes.add(mesh);

        let right_paddle_material = ColorMaterial::from(Color::rgb(0., 1., 0.));
        let left_paddle_material = ColorMaterial::from(Color::rgb(0., 0., 1.));

        commands.spawn((
            Player,
            PaddleBundle::new(right_paddle_x, 0.),
            MaterialMesh2dBundle {
                mesh: mesh_handle.clone().into(),
                material: materials.add(right_paddle_material),
                ..default()
            },
        ));

        commands.spawn((
            PaddleBundle::new(left_paddle_x, 0.),
            MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material: materials.add(left_paddle_material),
                ..default()
            },
        ));
    }
}

fn spawn_gutters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        let window_height = window.resolution.height();
        let top_gutter_y = window_height / 2. - GUTTER_HEIGHT / 2.;
        let bottom_gutter_y = -window_height / 2. + GUTTER_HEIGHT / 2.;

        let top_gutter = GutterBundle::new(0., top_gutter_y, window_width);
        let bottom_gutter = GutterBundle::new(0., bottom_gutter_y, window_width);
        let mesh = meshes.add(Mesh::from(shape::Quad::new(top_gutter.shape.0)));
        let material = materials.add(ColorMaterial::from(Color::rgb(0., 0., 0.)));

        commands.spawn((
            top_gutter,
            MaterialMesh2dBundle {
                mesh: mesh.clone().into(),
                material: material.clone(),
                ..default()
            },
        ));

        commands.spawn((
            bottom_gutter,
            MaterialMesh2dBundle {
                mesh: mesh.clone().into(),
                material: material.clone(),
                ..default()
            },
        ));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty().insert(Camera2dBundle::default());
}   