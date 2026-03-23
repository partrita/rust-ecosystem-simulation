use bevy::prelude::*;

/// 전역적으로 사용되는 텍스처 아틀라스 핸들 리소스
#[derive(Resource)]
pub struct GlobalTextureHandle(pub Option<Handle<TextureAtlas>>);

/// 시뮬레이션의 실행 상태를 나타내는 열거형
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SimState {
    #[default]
    Loading,    // 자산 로딩 중
    InitSim,    // 시뮬레이션 초기화 중
    Simulating, // 시뮬레이션 진행 중
}

/// 시뮬레이션 제어 및 디스플레이 설정 리소스
#[derive(Resource)]
pub struct Settings {
    pub camera_follow_boid: bool,     // 카메라가 일반 Boid를 추적하는지 여부
    pub camera_follow_predator: bool, // 카메라가 포식자를 추적하는지 여부
    pub camera_clamp_center: bool,    // 카메라가 중앙(0,0)으로 자동 복귀하는지 여부
    pub enable_gizmos: bool,          // 시각적 보조 지표(인식 범위 등) 표시 여부
    pub show_plots: bool,             // 통계 그래프 표시 여부
    pub show_plot_settings: bool,     // 그래프 설정 UI 표시 여부
    pub plot_options: PlotOptions,    // 어떤 데이터를 그래프로 그릴지 설정
    pub sim_speed: f32,               // 시뮬레이션 속도 배수 (1.0, 2.0, 4.0 등)
}

/// 그래프에 표시할 데이터 항목들
pub struct PlotOptions {
    pub num_boids: bool,    // 개체 수
    pub lifespan: bool,     // 평균 수명
    pub perception: bool,   // 평균 인식 범위
    pub affinity: bool,     // 먹이/독 선호도
    pub steering: bool,     // 조향 힘
    pub speed: bool,        // 이동 속도
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            camera_follow_boid: false,
            camera_follow_predator: false,
            camera_clamp_center: true,
            enable_gizmos: false,
            show_plots: false,
            show_plot_settings: false,
            plot_options: PlotOptions::default(),
            sim_speed: 1.0,
        }
    }
}

impl Default for PlotOptions {
    fn default() -> Self {
        Self {
            num_boids: false,
            lifespan: true,
            perception: true,
            affinity: true,
            steering: false,
            speed: false,
        }
    }
}
