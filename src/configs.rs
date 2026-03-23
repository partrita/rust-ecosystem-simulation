// 윈도우(화면) 설정
pub const WW: usize = 1600; // 초기 화면 가로 너비
pub const WH: usize = 1000; // 초기 화면 세로 높이

// 스프라이트(그래픽) 관련
pub const SPRITE_SHEET_PATH: &str = "assets.png"; // 스프라이트 시트 경로
pub const TILE_H: f32 = 16.0;                    // 각 스프라이트 타일 높이
pub const TILE_W: f32 = 16.0;                    // 각 스프라이트 타일 너비
pub const SPRITE_SHEET_ROWS: usize = 10;
pub const SPRITE_SHEET_COLS: usize = 10;
pub const BOID_SPRITE_SCALE: f32 = 1.5;          // Boid(생명체) 크기 배율
pub const CONSUMABLE_SPRITE_SCALE: f32 = 1.0;    // 먹이/독 크기 배율

// 시뮬레이션 환경 설정
pub const WORLD_W: f32 = 1305.0;  // 월드 가로 반경 (중앙 기준)
pub const WORLD_H: f32 = 730.0;   // 월드 세로 반경 (중앙 기준)
pub const NUM_BOIDS: usize = 100; // 초기 Boid 개수
pub const NUM_PREDATORS: usize = 10; // 초기 포식자 개수
pub const NUM_FOOD: usize = 600;     // 초기 먹이 개수
pub const NUM_POISON: usize = 150;   // 초기 독 개수

// Boid(보이드) 설정
pub const BOID_COLLISION_RADIUS: f32 = 8.0;     // 먹이 인식 및 충돌 반경
pub const BOID_MAX_HEALTH: f32 = 100.0;        // 최대 체력
pub const BOID_TICK_DAMAGE: f32 = 4.0;         // 감소하는 기초 체력 (속도가 빠를수록 증가)
pub const BOID_REPLICATE_INTERVAL: f32 = 5.0;  // 번식 시도 간격 (초)
pub const BOID_REPLICATE_PROBABILITY: f32 = 0.2; // 번식 성공 확률
pub const BOID_MUTATION_THRESHOLD: f32 = 0.8;    // DNA 돌연변이가 발생할 확률 임계값

// 포식자 설정
pub const BOID_NUTRITION: f32 = 30.0;          // 포식자가 Boid를 먹었을 때 회복하는 체력
pub const PREDATOR_COLLISION_RADIUS: f32 = 10.0; // Boid 사냥 시 충돌 반경

// 색상 (RGB 튜플)
pub const COLOR_FOOD: (u8, u8, u8) = (142, 231, 112);      // 먹이 색상
pub const COLOR_POISON: (u8, u8, u8) = (235, 86, 75);     // 독 색상
pub const COLOR_BOID: (u8, u8, u8) = (255, 255, 255);      // 건강한 Boid 색상
pub const COLOR_PREDATOR: (u8, u8, u8) = (255, 236, 179);  // 건강한 포식자 색상
pub const COLOR_PREDATOR_LOW_HEALTH: (u8, u8, u8) = (255, 145, 102); // 굶주린 포식자 색상
pub const COLOR_BOID_LOW_HEALTH: (u8, u8, u8) = (102, 255, 227);     // 굶주린 Boid 색상
pub const COLOR_BACKGROUND: (u8, u8, u8) = (50, 62, 79);   // 배경색

// 소모품(먹이, 독) 설정
pub const FOOD_NUTRITION: f32 = 5.0;         // 기본 먹이 영양가
pub const POISON_DAMAGE: f32 = -30.0;        // 기본 독 데미지
pub const FOOD_DECAY_RATE: f32 = -0.1;       // 먹이가 시간이 지남에 따라 변하는 정도
pub const POISON_DECAY_RATE: f32 = 0.5;      // 독이 시간이 지남에 따라 변하는 정도
pub const REPLICATION_RADIUS_FOOD: f32 = 40.0; // 먹이 기제 번식 시 주변 거리
pub const REPLICATION_RADIUS_POISON: f32 = 5.0;
pub const CONSUMABLE_REPLICATION_RATE: f32 = 0.2; // 소모품 재생성 주기
pub const CONSUMABLE_DECAY_RATE: f32 = 1.5;
pub const REPLICATION_COOLDOWN: f32 = 2.0;

// 통계 관련
pub const STAT_COLLECTION_RATE: f32 = 1.0;  // 통계 수집 간격 (초)
pub const MAX_NUM_POINTS: usize = 8000;      // 그래프에 저장할 최대 데이터 포인트 개수

// 그래프 렌더링 관련
pub const PLOT_LINE_WIDTH: f32 = 1.5;
pub const PLOT_ASPECT_RATIO: f32 = 1.8;
