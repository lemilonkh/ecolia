use bevy::prelude::*;
use bevy_turborand::prelude::*;
use std::{collections::HashMap, f32::consts::PI, time::Duration};

use crate::{
    plants::PlantType, utils::clamp_unit, RngResource, ANIMATION_COUNT, BASE_VELOCITY,
    CLOSEST_PLANT_PROBABILITY, DRINK_DURATION, DRINK_ENERGY_GAIN, EAT_DURATION, EAT_ENERGY_GAIN,
    RUN_ENERGY_DRAIN, RUN_HUNGER_DRAIN, RUN_THIRST_DRAIN, STAGE_SIZE,
};

const ANIMALS: &[&str] = &["Alpaca", "Deer", "Fox", "Husky", "Stag", "Wolf"];

#[derive(Component)]
pub struct Animal {
    name: String,
}

#[derive(Component)]
pub struct Velocity(Vec3);

#[derive(Component, PartialEq, Debug)]
pub enum AnimalState {
    Idle,
    Running,
    Eating,
    Drinking,
    Attacking,
    Dead,
}

#[derive(Component)]
pub struct Vitality {
    health: f32,
    energy: f32,
    thirst: f32,
    hunger: f32,
}

impl Default for Vitality {
    fn default() -> Self {
        Vitality {
            health: 1.0,
            energy: 1.0,
            thirst: 1.0,
            hunger: 1.0,
        }
    }
}

#[derive(Component)]
pub struct Target(Vec3);

#[derive(Resource)]
pub struct Animations(HashMap<String, Vec<Handle<AnimationClip>>>);

#[derive(Component)]
pub struct WaitTimer(Timer);

pub fn add_animals(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut global_rng: ResMut<GlobalRng>,
) {
    let mut rng = RngComponent::from(&mut global_rng);
    commands.insert_resource(RngResource(RngComponent::from(&mut global_rng)));
    let mut animations: HashMap<String, Vec<Handle<AnimationClip>>> = HashMap::new();

    for animal in ANIMALS {
        let file_name = format!("animals/{}.glb", animal);
        // build animation graphs
        let mut animation_clips: Vec<Handle<AnimationClip>> = vec![];
        for i in 0..ANIMATION_COUNT {
            animation_clips.push(assets.load(format!("{}#Animation{}", file_name, i)));
        }
        animations.insert(animal.to_string(), animation_clips);

        let alpaca = assets.load(format!("{}#Scene0", file_name));
        commands.spawn((
            Animal {
                name: animal.to_string(),
            },
            SceneBundle {
                scene: alpaca,
                transform: Transform::from_xyz(rng.f32() * STAGE_SIZE, 0.0, rng.f32() * STAGE_SIZE),
                ..default()
            },
            Vitality::default(),
            Velocity(Vec3::new(0.0, 0.0, 5.0)),
            AnimalState::Running,
            RngComponent::from(&mut global_rng),
            Target(Vec3::new(
                rng.f32() * STAGE_SIZE,
                0.0,
                rng.f32() * STAGE_SIZE,
            )),
        ));
    }

    commands.insert_resource(Animations(animations));
}

// Once the scene is loaded, start the animations
pub fn setup_animations_once_loaded(
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parent_query: Query<&Parent>,
    animals: Query<&Animal>,
) {
    for (entity, mut player) in &mut players {
        let Ok(parent) = parent_query.get(entity) else {
            println!("Missing parent!");
            continue;
        };
        let Ok(grandparent) = parent_query.get(parent.get()) else {
            println!("Missing grandparent!");
            continue;
        };
        let Ok(animal) = animals.get(grandparent.get()) else {
            println!("Missing animals!");
            continue;
        };

        println!(
            "Anims {}: Count {}",
            animal.name,
            animations.0[&animal.name].len()
        );
        player
            .play(animations.0[&animal.name][4].clone_weak())
            .repeat();
        player.seek_to(entity.index() as f32 / 10.0);
    }
}

pub fn process_wait_timer(
    time: Res<Time>,
    mut query: Query<(
        &mut WaitTimer,
        &Transform,
        &mut RngComponent,
        &mut AnimalState,
        &mut Target,
    )>,
    plants: Query<(Entity, &Transform, &PlantType)>,
    mut commands: Commands,
) {
    for (mut timer, transform, mut rng, mut state, mut target) in &mut query {
        if timer.0.tick(time.delta()).just_finished() {
            let mut min_dist = f32::INFINITY;
            let mut closest_plant: Option<Vec3> = None;
            if rng.f32() < CLOSEST_PLANT_PROBABILITY {
                for (entity, plant_transform, _plant_type) in &plants {
                    if plant_transform.translation.distance_squared(target.0) < 0.1 {
                        commands.entity(entity).despawn_recursive();
                        continue;
                    }
                    let dist = plant_transform
                        .translation
                        .distance_squared(transform.translation);
                    if dist < min_dist {
                        min_dist = dist;
                        closest_plant = Some(plant_transform.translation);
                    }
                }
            } else {
                let transforms = plants
                    .iter()
                    .map(|(_, transform, _)| transform)
                    .collect::<Vec<_>>();
                let random_index = rng.usize(0..transforms.len());
                closest_plant = Some(transforms[random_index].translation);
            }

            let Some(target_pos) = closest_plant else {
                continue;
            };

            target.0 = target_pos;
            *state = AnimalState::Running;
        }
    }
}

pub fn find_velocity(
    mut query: Query<
        (
            Entity,
            &mut Velocity,
            &mut Transform,
            &mut AnimalState,
            &Vitality,
            &Target,
        ),
        With<Animal>,
    >,
    mut commands: Commands,
) {
    for (entity, mut velocity, mut transform, mut state, vitality, target) in &mut query {
        if *state != AnimalState::Running {
            continue;
        }

        let to_target = target.0 - transform.translation;
        if to_target.length() < 0.8 {
            *state = AnimalState::Eating;

            // add WaitTimer component to entity, execute this in timer system when done
            commands
                .entity(entity)
                .insert(WaitTimer(Timer::from_seconds(5.0, TimerMode::Once)));
        }

        *state = AnimalState::Running;
        velocity.0 = to_target.normalize();
        velocity.0 *= BASE_VELOCITY * vitality.energy;
        transform.look_to(velocity.0, Vec3::Y);
        transform.rotate_local_y(PI);
    }
}

pub fn update_animal_animations(
    mut players: Query<(Entity, &mut AnimationPlayer)>,
    query: Query<(&AnimalState, &Animal, &Vitality)>,
    animations: Res<Animations>,
    parent_query: Query<&Parent>,
) {
    for (entity, mut player) in &mut players {
        let Ok(parent) = parent_query.get(entity) else {
            println!("Missing parent!");
            continue;
        };
        let Ok(grandparent) = parent_query.get(parent.get()) else {
            println!("Missing grandparent!");
            continue;
        };
        let Ok(animal_components) = query.get(grandparent.get()) else {
            println!("Missing animal components!");
            continue;
        };
        let (state, animal, vitality) = animal_components;

        let animation_index: usize = match state {
            AnimalState::Idle => 6,
            AnimalState::Running => 4,
            AnimalState::Eating => 11,
            _ => 8,
        };
        player
            .play_with_transition(
                animations.0[&animal.name][animation_index].clone_weak(),
                Duration::from_millis(250),
            )
            .repeat();
        player.set_speed(vitality.energy);
    }
}

pub fn update_animals(
    mut query: Query<(&mut Transform, &mut Vitality, &Velocity, &mut AnimalState)>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    for (mut transform, mut vitality, velocity, mut state) in &mut query {
        if vitality.health <= 0.0 {
            *state = AnimalState::Dead;
            return;
        }

        if *state == AnimalState::Running && velocity.0.length() > 0.0 {
            transform.translation += velocity.0 * delta;
            vitality.energy = clamp_unit(vitality.energy - RUN_ENERGY_DRAIN * delta);
            if vitality.energy == 0.0 {
                *state = AnimalState::Idle;
            }
            vitality.hunger = clamp_unit(vitality.hunger - RUN_HUNGER_DRAIN * delta);
            vitality.thirst = clamp_unit(vitality.thirst - RUN_THIRST_DRAIN * delta);
        }

        if *state == AnimalState::Eating {
            vitality.energy = clamp_unit(vitality.energy + EAT_ENERGY_GAIN * delta);
            vitality.hunger = clamp_unit(vitality.hunger - EAT_DURATION * delta);
        }

        if *state == AnimalState::Drinking {
            vitality.energy = clamp_unit(vitality.energy + DRINK_ENERGY_GAIN * delta);
            vitality.thirst = clamp_unit(vitality.thirst - DRINK_DURATION * delta);
        }
    }
}
