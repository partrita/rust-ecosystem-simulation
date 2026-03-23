use std::f32::consts::PI;
use std::time::Duration;

use bevy::math::vec3;
use bevy::prelude::*;
use bevy::utils::Instant;
use rand::Rng;

use crate::elements::{Consumable, Food, FoodBundle, Poison, PoisonBundle};
use crate::utils::*;
use crate::*;

/// Boid(생명체) 관련 로직을 관리하는 플러그인
pub struct BoidPlugin;

/// 일반 Boid 마커 컴포넌트
#[derive(Component)]
pub struct Boid;

/// 포식자 마커 컴포넌트
#[derive(Component)]
pub struct Predator;

/// 현재 이동 속도 (벡터)
#[derive(Component)]
struct Velocity(Vec2);

/// 현재 가속도 (벡터)
#[derive(Component)]
struct Acceleration(Vec2);

/// 개체의 현재 체력
#[derive(Component)]
struct Health(f32);

/// 번식 시도 주기를 관리하는 타이머
#[derive(Component)]
struct ReplicateTimer(Timer);

/// 개체가 태어난 시점
#[derive(Component)]
pub struct BirthTimeStamp(pub Instant);

/// 개체가 죽었을 때 먹이가 생성되는 이벤트
#[derive(Event)]
struct BoidDeathFoodSpawnEvent(Vec2);

/// Boid의 유전 정보 (DNA)
/// 각 값에 따라 개체의 행동 방식이 결정됩니다.
#[derive(Component, Clone, Copy)]
pub struct Dna {
    pub steering_force: f32,          // 조향 힘 (방향 전환 능력)
    pub speed: f32,                   // 이동 속도
    pub food_pull: f32,               // 먹이에 대한 선호도 (양수면 끌림, 음수면 기피)
    pub poison_pull: f32,             // 독에 대한 선호도
    pub predator_pull: f32,           // 포식자에 대한 반응 (보통 기피)
    pub food_perception_radius: f32,   // 먹이 인식 범위
    pub poison_perception_radius: f32, // 독 인식 범위
    pub predator_perception_radius: f32, // 포식자 인식 범위
}

/// 포식자 전용 유전 정보
#[derive(Component, Clone, Copy)]
pub struct PredatorDna {
    pub prey_perception: f32, // 사냥감(Boid) 인식 범위
    pub prey_pull: f32,       // 사냥감에 대한 추적 강도
}

/// Boid 개체를 생성하기 위한 번들
#[derive(Bundle)]
pub struct BoidBundle {
    sprite_sheet_bundle: SpriteSheetBundle,
    boid: Boid,
    velocity: Velocity,
    acceleration: Acceleration,
    dna: Dna,
    health: Health,
    replicate_timer: ReplicateTimer,
    birth_ts: BirthTimeStamp,
}

/// 포식자 개체를 생성하기 위한 번들
#[derive(Bundle)]
pub struct PredatorBundle {
    boid_bundle: BoidBundle,
    predator: Predator,
    predator_dna: PredatorDna,
}

// ----------------------------------------------------------------------------
// Implementations
// ----------------------------------------------------------------------------

impl BoidBundle {
    /// 무작위 위치에 새로운 Boid를 생성하는 설정을 반환합니다.
    pub fn new(handle: Handle<TextureAtlas>, is_predator: bool) -> Self {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-WORLD_W..WORLD_W);
        let y = rng.gen_range(-WORLD_H..WORLD_H);

        Self::child((x, y), &Dna::rand(), handle, is_predator)
    }

    /// 부모의 위치와 DNA를 기반으로 자식 Boid 설정을 반환합니다.
    fn child(pos: (f32, f32), dna: &Dna, handle: Handle<TextureAtlas>, is_predator: bool) -> Self {
        let (x, y) = pos;
        let sprite_index = if is_predator { 2 } else { 0 };
        Self {
            sprite_sheet_bundle: SpriteSheetBundle {
                texture_atlas: handle,
                sprite: TextureAtlasSprite::new(sprite_index),
                transform: Transform::from_scale(Vec3::splat(BOID_SPRITE_SCALE))
                    .with_translation(vec3(x, y, 2.0)),
                ..default()
            },
            boid: Boid,
            velocity: Velocity(get_rand_unit_vec2()),
            acceleration: Acceleration(Vec2::ZERO),
            dna: dna.clone(),
            health: Health(BOID_MAX_HEALTH),
            replicate_timer: ReplicateTimer(Timer::new(
                Duration::from_secs_f32(BOID_REPLICATE_INTERVAL),
                TimerMode::Repeating,
            )),
            birth_ts: BirthTimeStamp(Instant::now()),
        }
    }
}

impl PredatorBundle {
    /// 무작위 위치에 새로운 포식자를 생성합니다.
    pub fn new(handle: Handle<TextureAtlas>) -> Self {
        Self {
            boid_bundle: BoidBundle::new(handle, true),
            predator: Predator,
            predator_dna: PredatorDna::rand(),
        }
    }

    /// 부모 포식자로부터 전해진 DNA를 기반으로 자식을 생성합니다.
    fn child(
        pos: (f32, f32),
        dna: &Dna,
        predator_dna: &PredatorDna,
        handle: Handle<TextureAtlas>,
    ) -> Self {
        Self {
            boid_bundle: BoidBundle::child(pos, dna, handle, true),
            predator: Predator,
            predator_dna: predator_dna.clone(),
        }
    }
}

impl Dna {
    /// 무작위 DNA 생성
    fn rand() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            steering_force: rng.gen_range(0.0005..0.005),
            speed: rng.gen_range(0.5..1.5),
            food_pull: rng.gen_range(-1.5..1.5),
            poison_pull: rng.gen_range(-1.5..1.5),
            predator_pull: rng.gen_range(-1.5..1.5),
            food_perception_radius: rng.gen_range(50.0..120.0),
            poison_perception_radius: rng.gen_range(50.0..120.0),
            predator_perception_radius: rng.gen_range(50.0..120.0),
        }
    }

    /// 돌연변이 발생을 위한 수치 변화량 계산
    fn get_rand_mutation_delta() -> f32 {
        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0..1.0) > BOID_MUTATION_THRESHOLD {
            if rng.gen_range(0.0..1.0) > 0.5 {
                0.1
            } else {
                -0.1
            }
        } else {
            0.0
        }
    }

    /// 유전자 돌연변이 적용 (자식 세대에서 호출)
    fn mutate(&mut self) {
        self.steering_force += Self::get_rand_mutation_delta() * 0.01;
        self.speed += Self::get_rand_mutation_delta();
        self.food_pull += Self::get_rand_mutation_delta();
        self.poison_pull += Self::get_rand_mutation_delta();
        self.predator_pull += Self::get_rand_mutation_delta();
        self.food_perception_radius += Self::get_rand_mutation_delta() * 50.0;
        self.poison_perception_radius += Self::get_rand_mutation_delta() * 50.0;
    }
}

impl PredatorDna {
    fn rand() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            prey_perception: rng.gen_range(30.0..100.0),
            prey_pull: rng.gen_range(-1.0..1.0),
        }
    }

    fn mutate(&mut self) {
        self.prey_perception += Dna::get_rand_mutation_delta() * 50.0;
        self.prey_pull += Dna::get_rand_mutation_delta();
    }
}

impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_boid_transform,            // 위치 업데이트
                boundary_boids_direction_update, // 경계선 충돌 처리
                (update_boid_color, update_predator_color), // 체력에 따른 색상 변화
                (update_boid_direction, update_predator_direction), // 조향 계산
                (handle_boid_collision, handle_predator_collision), // 충돌(먹기) 처리
                (boids_replicate, predators_replicate), // 번식
                (camera_follow_boid, camera_follow_predator), // 카메라 추적
                despawn_boids,      // 죽은 개체 제거
                boid_health_tick,   // 시간에 따른 체력 감소
                handle_boid_despawn_events, // 죽었을 때 먹이 생성
                boid_separation,    // 개체 간 충돌 방지 (separation)
            )
                .run_if(in_state(SimState::Simulating)),
        )
        .add_event::<BoidDeathFoodSpawnEvent>();
    }
}

/// 일반 Boid의 번식 시스템
fn boids_replicate(
    time: Res<Time>,
    settings: Res<Settings>,
    mut commands: Commands,
    handle: Res<GlobalTextureHandle>,
    mut boid_query: Query<
        (&mut ReplicateTimer, &Transform, &Dna, &Health),
        (With<Boid>, Without<Predator>),
    >,
) {
    if boid_query.iter().len() >= NUM_BOIDS {
        return;
    }

    let mut rng = rand::thread_rng();
    let replication_threshold = 1.0 - BOID_REPLICATE_PROBABILITY;
    
    let scaled_delta = time.delta().mul_f32(settings.sim_speed);

    for (mut timer, transform, dna, health) in boid_query.iter_mut() {
        // 체력이 충분할 때만 일정 확률로 번식
        if rng.gen_range(0.0..1.0) <= replication_threshold && health.0 > BOID_MAX_HEALTH / 2.0 {
            continue;
        }
        
        if !timer.0.tick(scaled_delta).just_finished() {
            continue;
        }

        let (x, y) = (transform.translation.x, transform.translation.y);
        let mut child_dna = dna.clone();
        child_dna.mutate();

        // 동일한 위치에 자식 스폰
        commands.spawn(BoidBundle::child(
            (x, y),
            &child_dna,
            handle.0.clone().unwrap(),
            false,
        ));
    }
}

/// 포식자의 번식 시스템
fn predators_replicate(
    time: Res<Time>,
    settings: Res<Settings>,
    mut commands: Commands,
    handle: Res<GlobalTextureHandle>,
    mut predators_query: Query<
        (&mut ReplicateTimer, &Transform, &Dna, &PredatorDna, &Health),
        With<Predator>,
    >,
) {
    if predators_query.iter().len() >= NUM_PREDATORS {
        return;
    }

    let mut rng = rand::thread_rng();
    let replication_threshold = 1.0 - BOID_REPLICATE_PROBABILITY;
    
    let scaled_delta = time.delta().mul_f32(settings.sim_speed);

    for (mut timer, transform, dna, predator_dna, health) in predators_query.iter_mut() {
        if rng.gen_range(0.0..1.0) <= replication_threshold && health.0 > BOID_MAX_HEALTH / 3.0 {
            continue;
        }
        
        if !timer.0.tick(scaled_delta).just_finished() {
            continue;
        }

        let (x, y) = (transform.translation.x, transform.translation.y);
        let mut child_dna = dna.clone();
        let mut child_predator_dna = predator_dna.clone();
        child_dna.mutate();
        child_predator_dna.mutate();

        commands.spawn(PredatorBundle::child(
            (x, y),
            &child_dna,
            &child_predator_dna,
            handle.0.clone().unwrap(),
        ));
    }
}

/// 설정에 따라 카메라가 포식자를 따라가도록 함
fn camera_follow_predator(
    predator_query: Query<&Transform, With<Predator>>,
    settings: Res<Settings>,
    mut cam_query: Query<&mut Transform, (With<Camera>, Without<Predator>)>,
) {
    if cam_query.is_empty() || predator_query.is_empty() || !settings.camera_follow_predator {
        return;
    }

    let mut cam_transform = cam_query.single_mut();
    if let Some(transform) = predator_query.iter().next() {
        let mut new_pos = transform.translation;
        new_pos.y -= 50.0;
        cam_transform.translation = cam_transform.translation.lerp(new_pos, 0.05);
    }
}

/// 설정에 따라 카메라가 Boid를 따라가도록 함
fn camera_follow_boid(
    boid_query: Query<&Transform, (With<Boid>, Without<Predator>)>,
    settings: Res<Settings>,
    mut cam_query: Query<&mut Transform, (With<Camera>, Without<Boid>)>,
) {
    if cam_query.is_empty() || boid_query.is_empty() || !settings.camera_follow_boid {
        return;
    }

    let mut cam_transform = cam_query.single_mut();
    if let Some(transform) = boid_query.iter().next() {
        let mut new_pos = transform.translation;
        new_pos.y -= 50.0;
        cam_transform.translation = cam_transform.translation.lerp(new_pos, 0.05);
    }
}

/// 물리 및 위치 업데이트 시스템 (가속도 -> 속도 -> 위치)
fn update_boid_transform(
    settings: Res<Settings>,
    mut boids_query: Query<(&mut Transform, &mut Velocity, &mut Acceleration, &Dna), With<Boid>>,
) {
    if boids_query.is_empty() {
        return;
    }

    for (mut transform, mut velocity, mut acceleration, dna) in boids_query.iter_mut() {
        let old_pos = transform.translation;
        if acceleration.0.is_nan() {
            acceleration.0 = Vec2::ZERO;
            continue;
        }

        // 1. 속도 업데이트 (가속도 반영 및 정규화)
        velocity.0 = (velocity.0 + acceleration.0 * settings.sim_speed).normalize();
        acceleration.0 = Vec2::ZERO;

        // 2. 위치 업데이트 (DNA 속도 및 시뮬레이션 속도 반영)
        let speed_multiplier = dna.speed * settings.sim_speed;
        let new_translation =
            transform.translation + vec3(velocity.0.x, velocity.0.y, 0.0) * speed_multiplier;
        if !new_translation.is_nan() {
            transform.translation = new_translation;
        }

        // 3. 회전 설정 (이동 방향을 향하도록)
        transform.rotation =
            Quat::from_rotation_z(calc_rotation_angle(old_pos, transform.translation) + PI / 2.0);
    }
}

/// Boid가 죽었을 때 무작위로 먹이나 독을 생성하는 이벤트 처리
fn handle_boid_despawn_events(
    mut commands: Commands,
    handle: Res<GlobalTextureHandle>,
    mut events: EventReader<BoidDeathFoodSpawnEvent>,
) {
    let mut rng = rand::thread_rng();
    for e in events.read() {
        for _ in 0..rng.gen_range(2..5) {
            let (x, y) = (e.0.x, e.0.y);
            let (x, y) = (
                x + rng.gen_range(-30.0..30.0),
                y + rng.gen_range(-30.0..30.0),
            );

            let (x, y) = limit_to_world((x, y));
            if rng.gen_range(0.0..1.0) > 0.85 {
                commands.spawn(PoisonBundle::new((x, y), handle.0.clone().unwrap()));
            } else {
                commands.spawn(FoodBundle::new((x, y), handle.0.clone().unwrap()));
            }
        }
    }
}

/// Boid와 소모품(먹이, 독) 간의 충돌 및 효과 처리
fn handle_boid_collision(
    mut commands: Commands,
    mut boids_query: Query<(&mut Health, &Transform), (With<Boid>, Without<Predator>)>,
    consumable_query: Query<(&Transform, &Consumable, Entity), With<Consumable>>,
) {
    let mut consumables = Vec::new();
    for (t, c, e) in consumable_query.iter() {
        consumables.push((t.translation, c.0, e));
    }

    for (mut health, boid_transform) in boids_query.iter_mut() {
        let mut food_idx = None;
        for (index, (t, c, e)) in consumables.iter().enumerate() {
            if boid_transform.translation.distance_squared(*t)
                < BOID_COLLISION_RADIUS * BOID_COLLISION_RADIUS
            {
                // 체력 회복/데미지 적용 및 소모품 제거
                health.0 = (health.0 + c).min(BOID_MAX_HEALTH);
                commands.entity(*e).despawn();
                food_idx = Some(index);
                break;
            }
        }
        if let Some(index) = food_idx {
            consumables.remove(index);
        }
    }
}

/// 포식자와 일반 Boid 간의 충돌(사냥) 처리
fn handle_predator_collision(
    mut commands: Commands,
    mut writer: EventWriter<BoidDeathFoodSpawnEvent>,
    mut predator_query: Query<(&mut Health, &Transform), With<Predator>>,
    boids_query: Query<(&Transform, Entity), (With<Boid>, Without<Predator>)>,
) {
    let mut boids = Vec::new();
    for (t, e) in boids_query.iter() {
        boids.push((t.translation, e));
    }

    for (mut health, predator_transform) in predator_query.iter_mut() {
        let mut boid_idx = None;
        for (index, (t, e)) in boids.iter().enumerate() {
            if predator_transform.translation.distance_squared(*t)
                < PREDATOR_COLLISION_RADIUS * PREDATOR_COLLISION_RADIUS
            {
                // 포식자 체력 회복 및 Boid 제거, 파편 생성 이벤트 발송
                health.0 = (health.0 + BOID_NUTRITION).min(BOID_MAX_HEALTH);
                writer.send(BoidDeathFoodSpawnEvent(t.truncate()));
                boid_idx = Some(index);
                commands.entity(*e).despawn();
                break;
            }
        }

        if let Some(index) = boid_idx {
            boids.remove(index);
        }
    }
}

/// 체력이 0이 된 Boid를 월드에서 제거
fn despawn_boids(
    mut commands: Commands,
    mut writer: EventWriter<BoidDeathFoodSpawnEvent>,
    boids_query: Query<(&Health, &Transform, Entity), With<Boid>>,
) {
    for (health, transform, e) in boids_query.iter() {
        if health.0 <= 0.0 {
            commands.entity(e).despawn();
            writer.send(BoidDeathFoodSpawnEvent(transform.translation.truncate()));
        }
    }
}

/// 월드 시간에 흐름에 따른 체력 소모 시스템
fn boid_health_tick(
    time: Res<Time>,
    settings: Res<Settings>,
    mut boids_query: Query<(&mut Health, &Dna), With<Boid>>,
) {
    let scaled_delta = time.delta_seconds() * settings.sim_speed;
    
    for (mut health, dna) in boids_query.iter_mut() {
        // 속도가 빠를수록 체력이 더 빨리 소모됨
        let frame_damage = BOID_TICK_DAMAGE * dna.speed.max(1.0) * 0.5 * (scaled_delta / 0.5);
        health.0 -= frame_damage;
    }
}

/// 월드 경계선을 벗어나지 않도록 조향 힘을 조절
fn boundary_boids_direction_update(
    mut boids_query: Query<(&Transform, &Velocity, &mut Acceleration), With<Boid>>,
) {
    if boids_query.is_empty() {
        return;
    }

    for (transform, velocity, mut acceleration) in boids_query.iter_mut() {
        let (x, y) = (transform.translation.x, transform.translation.y);
        let (min_x, min_y, max_x, max_y) = get_world_bounds();
        if x <= min_x || x >= max_x || y <= min_y || y >= max_y {
            // 중앙(0,0) 방향으로 돌아가는 조향 힘 적용
            acceleration.0 +=
                get_steering_force(Vec2::ZERO, transform.translation.truncate(), velocity.0)
                    * 0.001;
        }
    }
}

/// 포식자의 사냥감(Boid) 추적 조향 계산
fn update_predator_direction(
    mut predator_query: Query<
        (&Transform, &Velocity, &mut Acceleration, &Dna, &PredatorDna),
        With<Predator>,
    >,
    boids_query: Query<&Transform, (With<Boid>, Without<Predator>)>,
) {
    if predator_query.is_empty() || boids_query.is_empty() {
        return;
    }

    for (predator_transform, velocity, mut acceleration, dna, predator_dna) in
        predator_query.iter_mut()
    {
        let mut closest_boid_dist = f32::INFINITY;
        let mut closest_boid_pos = None;
        for boid_transform in boids_query.iter() {
            let dist = predator_transform
                .translation
                .distance_squared(boid_transform.translation);
            if dist <= closest_boid_dist
                && dist <= predator_dna.prey_perception * predator_dna.prey_perception
            {
                closest_boid_dist = dist;
                closest_boid_pos = Some(boid_transform.translation.truncate());
            }
        }

        if let Some(boid_pos) = closest_boid_pos {
            acceleration.0 += get_steering_force(
                boid_pos,
                predator_transform.translation.truncate(),
                velocity.0,
            ) * dna.steering_force.abs()
                * predator_dna.prey_pull;
        }
    }
}

/// Boid들이 서로 너무 겹치지 않게 조절하는 분리(Separation) 시스템
fn boid_separation(mut boids_query: Query<(&Transform, &Velocity, &mut Acceleration), With<Boid>>) {
    if boids_query.is_empty() {
        return;
    }

    let all_positions: Vec<Vec2> = boids_query
        .into_iter()
        .map(|(t, _, _)| t.translation.truncate())
        .collect();

    for (transform, velocity, mut acceleration) in boids_query.iter_mut() {
        let boid_pos = transform.translation.truncate();
        for neighbour_pos in all_positions.iter() {
            let dist = neighbour_pos.distance_squared(boid_pos);
            // 일정 거리 안의 이웃이 있으면 반대 방향으로 힘 적용
            if dist != 0.0 && dist <= 20.0 {
                acceleration.0 += get_steering_force(*neighbour_pos, boid_pos, velocity.0) * -0.01;
            }
        }
    }
}

/// 일반 Boid의 먹이 추적 및 독/포식자 회피 조향 계산
fn update_boid_direction(
    mut boids_query: Query<
        (&Transform, &Velocity, &mut Acceleration, &Dna),
        (With<Boid>, Without<Predator>),
    >,
    predator_query: Query<&Transform, With<Predator>>,
    food_query: Query<&Transform, With<Food>>,
    poison_query: Query<&Transform, With<Poison>>,
) {
    if boids_query.is_empty() {
        return;
    }

    for (transform, velocity, mut acceleration, dna) in boids_query.iter_mut() {
        let closest_food = get_closest_item(
            transform.translation,
            dna.food_perception_radius,
            food_query.iter(),
        );
        let closest_poison = get_closest_item(
            transform.translation,
            dna.poison_perception_radius,
            poison_query.iter(),
        );
        let closest_predator = get_closest_item(
            transform.translation,
            dna.predator_perception_radius,
            predator_query.iter(),
        );

        // 가장 가까운 먹이 방향으로 힘 적용
        if let Some(food) = closest_food {
            acceleration.0 +=
                get_steering_force(food, transform.translation.truncate(), velocity.0)
                    * dna.steering_force.abs()
                    * dna.food_pull;
        }
        // 가장 가까운 독 회피
        if let Some(poison) = closest_poison {
            acceleration.0 +=
                get_steering_force(poison, transform.translation.truncate(), velocity.0)
                    * dna.steering_force.abs()
                    * dna.poison_pull;
        }
        // 포식자 회피
        if let Some(predator) = closest_predator {
            acceleration.0 +=
                get_steering_force(predator, transform.translation.truncate(), velocity.0)
                    * dna.steering_force.abs()
                    * dna.predator_pull;
        }
    }
}

/// 인식 범위 내의 가장 가까운 항목의 위치를 찾음
fn get_closest_item<'a, I>(origin: Vec3, range: f32, items: I) -> Option<Vec2>
where
    I: IntoIterator<Item = &'a Transform>,
{
    let mut closest_item_dist = f32::INFINITY;
    let mut closest_item_pos = None;

    for t in items {
        let dist = t.translation.distance_squared(origin);
        if dist <= closest_item_dist && dist <= range * range {
            closest_item_dist = dist;
            closest_item_pos = Some(t.translation.truncate());
        }
    }

    closest_item_pos
}

/// 체력에 따라 Boid의 색상을 변경 (건강함: 흰색, 굶주림: 녹색빛)
fn update_boid_color(
    mut boids_query: Query<(&mut TextureAtlasSprite, &Health), (With<Boid>, Without<Predator>)>,
) {
    for (mut sprite, health) in boids_query.iter_mut() {
        sprite.color = Color::from(
            Vec4::from(get_color(COLOR_BOID_LOW_HEALTH))
                .lerp(Vec4::from(get_color(COLOR_BOID)), health.0 / 100.0),
        );
    }
}

/// 체력에 따라 포식자의 색상을 변경 (건강함: 노란색, 굶주림: 주황색)
fn update_predator_color(
    mut predator_query: Query<(&mut TextureAtlasSprite, &Health), With<Predator>>,
) {
    for (mut sprite, health) in predator_query.iter_mut() {
        sprite.color = Color::from(
            Vec4::from(get_color(COLOR_PREDATOR_LOW_HEALTH))
                .lerp(Vec4::from(get_color(COLOR_PREDATOR)), health.0 / 100.0),
        );
    }
}


