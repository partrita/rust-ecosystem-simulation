use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use rand::Rng;

use crate::boid::{Boid, BoidBundle, Predator, PredatorBundle};
use crate::elements::{FoodBundle, PoisonBundle};
use crate::SimState;
use crate::*;

/// 월드의 초기화 및 엔티티 생성을 담당하는 플러그인
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            OnEnter(SimState::InitSim),
            (populate_boids, populate_consumables, start_simulation),
        )
        .add_systems(
            Update,
            populate_boids
                .run_if(on_timer(Duration::from_secs_f32(5.0)))
                .run_if(in_state(SimState::Simulating)),
        );
    }
}

/// 초기 Boid와 포식자 개체들을 월드에 스폰
fn populate_boids(
    mut commands: Commands,
    handle: Res<GlobalTextureHandle>,
    boid_query: Query<(With<Boid>, Without<Predator>)>,
    predator_query: Query<With<Predator>>,
) {
    // 이미 개체가 있다면 새로 생성하지 않음 (초기화 단계에서 사용)
    let remaining_boids = if boid_query.iter().len() > 0 {
        0
    } else {
        NUM_BOIDS
    };
    let remaining_predators = if predator_query.iter().len() > 0 {
        0
    } else {
        NUM_PREDATORS
    };

    for _ in 0..remaining_boids {
        commands.spawn(BoidBundle::new(handle.0.clone().unwrap(), false));
    }
    for _ in 0..remaining_predators {
        commands.spawn(PredatorBundle::new(handle.0.clone().unwrap()));
    }
}

/// 초기 먹이와 독 소모품을 월드에 스폰
fn populate_consumables(mut commands: Commands, handle: Res<GlobalTextureHandle>) {
    let mut rng = rand::thread_rng();

    for _ in 0..NUM_FOOD {
        let x = rng.gen_range(-WORLD_W..WORLD_W);
        let y = rng.gen_range(-WORLD_H..WORLD_H);
        commands.spawn(FoodBundle::new((x, y), handle.0.clone().unwrap()));
    }
    for _ in 0..NUM_POISON {
        let x = rng.gen_range(-WORLD_W..WORLD_W);
        let y = rng.gen_range(-WORLD_H..WORLD_H);
        commands.spawn(PoisonBundle::new((x, y), handle.0.clone().unwrap()));
    }
}

/// 모든 초기화가 완료되면 시뮬레이션 상태를 Simulating으로 변경
fn start_simulation(mut next_state: ResMut<NextState<SimState>>) {
    next_state.set(SimState::Simulating);
}

