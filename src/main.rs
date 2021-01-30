use bevy::{prelude::*, sprite::collide_aabb::collide};
use rand::Rng;

const GRAVITY_ACCELERATION: f32 = 9.8;

const WINDOW_HEIGHT: f32 = 720.0;
const WINDOW_WIDTH: f32 = 1280.0;
const PLAYER_HEIGHT: f32 = 120.0;
const FLOOR_POSITION_Y: f32 = -WINDOW_HEIGHT / 2.0 + PLAYER_HEIGHT / 2.0;
const BULLET_DIMENSION: f32 = 15.0;
const TARGET_DIMENSION: f32 = 50.0;
#[derive(Debug)]
struct Player {
    velocity: Vec3,
    speed: f32,
    direction: Direction,
}

#[derive(Debug)]
enum Direction {
    Right,
    Left,
}

#[derive(Debug)]
struct Bullet {
    velocity: Vec3,
    speed: f32,
}

#[derive(Debug)]
struct Target {
    velocity: Vec3,
    speed: f32,
}

struct TargetSpawnTimer(Timer);

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Shooter game".to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .add_resource(TargetSpawnTimer(Timer::from_seconds(1.0, true)))
        .add_startup_system(setup.system())
        .add_system(physics.system())
        .add_system(move_player.system())
        .add_system(player_action.system())
        .add_system(move_bullet.system())
        .add_system(target_spawner.system())
        .add_system(move_target.system())
        .add_system(target_collision.system())
        .run();
}

fn setup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default())
        // Player
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, WINDOW_HEIGHT / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(30.0, PLAYER_HEIGHT)),
            ..Default::default()
        })
        .with(Player {
            velocity: Vec3::new(0.0, 0.0, 0.0),
            speed: 400.0,
            direction: Direction::Right,
        });
}

fn physics(time: Res<Time>, mut query: Query<(&mut Player, &mut Transform)>) {
    for (mut player, mut transform) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.y += player.velocity.y;
        if translation.y > FLOOR_POSITION_Y {
            player.velocity.y -= time.delta_seconds() * GRAVITY_ACCELERATION;
        }
        if player.velocity.y < 0.0 && translation.y <= FLOOR_POSITION_Y {
            player.velocity.y = 0.0;
        }
    }
}

fn move_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    for (mut player, mut transform) in query.iter_mut() {
        let translation = &mut transform.translation;
        if (keyboard_input.just_pressed(KeyCode::Up) || keyboard_input.just_pressed(KeyCode::W))
            && (translation.y <= FLOOR_POSITION_Y)
        {
            player.velocity.y = 10.0;
        }

        let mut direction = 0.0;
        for key in keyboard_input.get_pressed() {
            match key {
                KeyCode::Left => {
                    direction -= 1.0;
                    player.direction = Direction::Left;
                }
                KeyCode::Right => {
                    direction += 1.0;
                    player.direction = Direction::Right;
                }
                KeyCode::A => {
                    direction -= 1.0;
                    player.direction = Direction::Left;
                }
                KeyCode::D => {
                    direction += 1.0;
                    player.direction = Direction::Right;
                }
                _ => (),
            }
        }
        translation.x += time.delta_seconds() * direction * player.speed;
    }
}

fn move_bullet(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(&Bullet, &mut Transform, Entity)>,
) {
    for (bullet, mut transform, entity) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x += time.delta_seconds() * bullet.velocity.x * bullet.speed;
        translation.y += time.delta_seconds() * bullet.velocity.y * bullet.speed;
        if translation.y <= -WINDOW_HEIGHT / 2.0
            || translation.y >= WINDOW_HEIGHT / 2.0
            || translation.x <= -WINDOW_WIDTH / 2.0
            || translation.x >= WINDOW_WIDTH / 2.0
        {
            commands.despawn(entity);
        }
    }
}

fn player_action(
    commands: &mut Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(&Player, &Transform)>,
) {
    for (player, player_transform) in query.iter() {
        if keyboard_input.just_pressed(KeyCode::Return) {
            let x = if let Direction::Left = player.direction {
                -1.0
            } else {
                1.0
            };
            commands
                .spawn(SpriteBundle {
                    material: materials.add(Color::rgb(1.0, 0.5, 0.0).into()),
                    transform: Transform::from_translation(player_transform.translation),
                    sprite: Sprite::new(Vec2::new(BULLET_DIMENSION, BULLET_DIMENSION)),
                    ..Default::default()
                })
                .with(Bullet {
                    velocity: Vec3::new(x, 0.0, 0.0),
                    speed: 500.0,
                });
        }
    }
}

fn move_target(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(&Target, &mut Transform, Entity)>,
) {
    for (target, mut transform, entity) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x += time.delta_seconds() * target.velocity.x * target.speed;
        if translation.x <= -WINDOW_WIDTH / 2.0 {
            commands.despawn(entity);
        }
    }
}

fn target_spawner(
    commands: &mut Commands,
    time: Res<Time>,
    mut timer: ResMut<TargetSpawnTimer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();
    timer.0.tick(time.delta_seconds());
    if timer.0.just_finished() {
        commands
            .spawn(SpriteBundle {
                material: materials.add(Color::rgb(0.125, 0.75, 0.0).into()),
                transform: Transform::from_translation(Vec3::new(
                    640.0,
                    rng.gen_range(FLOOR_POSITION_Y..0.0),
                    0.0,
                )),
                sprite: Sprite::new(Vec2::new(TARGET_DIMENSION, TARGET_DIMENSION)),
                ..Default::default()
            })
            .with(Target {
                velocity: Vec3::new(-1.0, 0.0, 0.0),
                speed: 100.0,
            });
    }
}

fn target_collision(
    commands: &mut Commands,
    bullet_query: Query<(&Bullet, Entity, &Transform, &Sprite)>,
    target_query: Query<(&Target, Entity, &Transform, &Sprite)>,
) {
    for (_bullet, bullet_entity, bullet_transform, bullet_sprite) in bullet_query.iter() {
        for (_target, target_entity, target_transform, target_sprite) in target_query.iter() {
            if collide(
                bullet_transform.translation,
                bullet_sprite.size,
                target_transform.translation,
                target_sprite.size,
            )
            .is_some()
            {
                commands.despawn(target_entity);
                commands.despawn(bullet_entity);
            }
        }
    }
}
