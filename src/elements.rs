use bevy::{math::vec3, prelude::*, utils::Instant};
use rand::Rng;

use crate::*;
use self::utils::{get_color, limit_to_world};

/// 먹이와 독 같은 소모성 요소들을 관리하는 플러그인
pub struct ElementsPlugin;

/// 소모품 재생성 및 부패를 위한 타이머 리소스
#[derive(Resource)]
struct SimulationTimers {
    replicate: Timer,
    decay: Timer,
}

impl Default for SimulationTimers {
    fn default() -> Self {
        Self {
            replicate: Timer::from_seconds(CONSUMABLE_REPLICATION_RATE, TimerMode::Repeating),
            decay: Timer::from_seconds(CONSUMABLE_DECAY_RATE, TimerMode::Repeating),
        }
    }
}

/// 소모품 컴포넌트 (f32 값은 영양가 또는 데미지 양)
#[derive(Component)]
pub struct Consumable(pub f32);

/// 먹이 마커 컴포넌트
#[derive(Component)]
pub struct Food;

/// 독 마커 컴포넌트
#[derive(Component)]
pub struct Poison;

/// 마지막으로 번식(자기복제)한 시점
#[derive(Component)]
struct LastReplicationTs(Instant);

/// 소모품 기본 번들
#[derive(Bundle)]
struct ConsumableBundle {
    sprite_sheet_bundle: SpriteSheetBundle,
    last_replication_ts: LastReplicationTs,
    consumable: Consumable,
}

/// 먹이 번들
#[derive(Bundle)]
pub struct FoodBundle {
    food: Food,
    consumable_bundle: ConsumableBundle,
}

/// 독 번들
#[derive(Bundle)]
pub struct PoisonBundle {
    poison: Poison,
    consumable_bundle: ConsumableBundle,
}

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationTimers>()
            .add_systems(
                Update,
                (
                    replicate_consumables, // 소모품 번식
                    decay_consumables,     // 소모품 부패
                    despawn_consumables,   // 수명이 다한 소모품 제거
                    update_color,          // 부패 정도에 따른 투명도 업데이트
                )
                    .run_if(in_state(SimState::Simulating)),
            );
    }
}

/// 소모품의 부패 정도(영양가)에 따라 투명도를 변경
fn update_color(
    mut consumable_query: Query<(&mut TextureAtlasSprite, &Consumable), With<Consumable>>,
) {
    for (mut sprite, c) in consumable_query.iter_mut() {
        sprite.color = sprite.color.with_a(c.opacity());
    }
}

/// 소모품이 시간이 지남에 따라 점차 사라지게(부패) 함
fn decay_consumables(
    time: Res<Time>,
    settings: Res<Settings>,
    mut timers: ResMut<SimulationTimers>,
    mut consumable_query: Query<&mut Consumable, With<Consumable>>,
) {
    if !timers.decay.tick(time.delta().mul_f32(settings.sim_speed)).just_finished() {
        return;
    }

    if consumable_query.is_empty() {
        return;
    }

    for mut c in consumable_query.iter_mut() {
        c.decay()
    }
}

/// 가치가 0에 가까워진 소모품을 제거
fn despawn_consumables(
    mut commands: Commands,
    consumable_query: Query<(&Consumable, Entity), With<Consumable>>,
) {
    for (c, e) in consumable_query.iter() {
        if c.is_despawn() {
            commands.entity(e).despawn();
        }
    }
}

/// 소모품(먹이, 독)이 주변으로 번지는 시스템
fn replicate_consumables(
    time: Res<Time>,
    settings: Res<Settings>,
    mut timers: ResMut<SimulationTimers>,
    mut commands: Commands,
    handle: Res<GlobalTextureHandle>,
    food_query: Query<With<Food>>,
    poison_query: Query<With<Poison>>,
    mut consumable_query: Query<
        (&Transform, &Consumable, &mut LastReplicationTs),
        With<Consumable>,
    >,
) {
    if !timers.replicate.tick(time.delta().mul_f32(settings.sim_speed)).just_finished() {
        return;
    }

    let num_food = food_query.iter().len();
    let num_poison = poison_query.iter().len();
    // 설정된 최대 개수보다 적을 때만 생성
    let is_populate_food = num_food < NUM_FOOD;
    let is_populate_poison = num_poison < NUM_POISON;

    let mut rng = rand::thread_rng();
    for (transform, consumable, mut last_replication_ts) in consumable_query.iter_mut() {
        // 개별 쿨타임 확인
        if last_replication_ts.0.elapsed().as_secs_f32() * settings.sim_speed < REPLICATION_COOLDOWN {
            continue;
        }
        if rng.gen_range(0.0..1.0) < 0.8 {
            continue;
        }

        // 월드 중앙에 가까울수록 번식 확률이 높음 (가장자리 개체 과잉 방지)
        let dist_to_center = transform
            .translation
            .truncate()
            .distance_squared(Vec2::ZERO);
        if rng.gen_range(0.0..1.0) < dist_to_center / 1_000_000.0 {
            continue;
        }

        let replication_radius = if consumable.is_food() {
            REPLICATION_RADIUS_FOOD
        } else {
            REPLICATION_RADIUS_POISON
        };
        let (x, y) = (transform.translation.x, transform.translation.y);
        let (nx, ny) = (
            x + rng.gen_range(-replication_radius..replication_radius),
            y + rng.gen_range(-replication_radius..replication_radius),
        );
        let (nx, ny) = limit_to_world((nx, ny));

        // 새로운 먹이/독 스폰
        if consumable.is_food() && is_populate_food {
            commands.spawn(FoodBundle::new((nx, ny), handle.0.clone().unwrap()));
        } else if rng.gen_range(0.0..1.0) > 0.4 && is_populate_poison {
            commands.spawn(PoisonBundle::new((nx, ny), handle.0.clone().unwrap()));
        }
        last_replication_ts.0 = Instant::now();
    }
}

impl Consumable {
    pub fn is_food(&self) -> bool {
        self.0 >= 0.1
    }

    /// 소멸 대상인지 확인
    fn is_despawn(&self) -> bool {
        (self.is_food() && self.0 <= 0.1) || (!self.is_food() && self.0 >= -0.1)
    }

    /// 시간에 따른 영양가/독성 수치 변화
    fn decay(&mut self) {
        if self.is_food() {
            self.0 = (self.0 + FOOD_DECAY_RATE).max(0.1);
            return;
        }

        self.0 = (self.0 + POISON_DECAY_RATE).min(-0.1);
    }

    /// 남은 수치에 따른 불투명도 계산
    fn opacity(&self) -> f32 {
        if self.is_food() {
            self.0 / FOOD_NUTRITION
        } else {
            self.0 / POISON_DAMAGE
        }
    }
}

impl ConsumableBundle {
    pub fn new(
        (x, y): (f32, f32),
        handle: Handle<TextureAtlas>,
        color: Color,
        nutrition: f32,
    ) -> Self {
        Self {
            sprite_sheet_bundle: SpriteSheetBundle {
                texture_atlas: handle,
                sprite: TextureAtlasSprite {
                    index: 1,
                    color,
                    ..default()
                },
                transform: Transform::from_scale(Vec3::splat(CONSUMABLE_SPRITE_SCALE))
                    .with_translation(vec3(x, y, 2.0)),
                ..default()
            },
            consumable: Consumable(nutrition),
            last_replication_ts: LastReplicationTs(Instant::now()),
        }
    }
}

impl FoodBundle {
    pub fn new((x, y): (f32, f32), handle: Handle<TextureAtlas>) -> Self {
        Self {
            consumable_bundle: ConsumableBundle::new(
                (x, y),
                handle,
                get_color(COLOR_FOOD),
                FOOD_NUTRITION,
            ),
            food: Food,
        }
    }
}

impl PoisonBundle {
    pub fn new((x, y): (f32, f32), handle: Handle<TextureAtlas>) -> Self {
        Self {
            consumable_bundle: ConsumableBundle::new(
                (x, y),
                handle,
                get_color(COLOR_POISON),
                POISON_DAMAGE,
            ),
            poison: Poison,
        }
    }
}

