use bevy::math::vec2;
use bevy::prelude::*;
use bevy::window::close_on_esc;
use bevy_pancam::{PanCam, PanCamPlugin};

use ecosim::utils::get_color;
use ecosim::*;
use ecosim::{
    boid::BoidPlugin, elements::ElementsPlugin, gui::GuiPlugin, stats::StatsPlugin,
    world::WorldPlugin,
};

/// 애플리케이션의 진입점으로, Bevy 앱을 설정하고 필요한 플러그인과 시스템을 등록합니다.
fn main() {
    App::new()
        // 시뮬레이션 상태(Loading, InitSim, Simulating)를 관리하기 위한 상태 추가
        .add_state::<SimState>()
        // 기본 Bevy 플러그인 설정 (윈도우 크기, 타이틀 등)
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (WW as f32, WH as f32).into(),
                        title: "ecosystem simulation".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        // 카메라 조작을 위한 PanCam 플러그인
        .add_plugins(PanCamPlugin)
        // 리소스 설정: MSAA 비활성화, 배경색, 텍스처 핸들, 초기 설정값 등
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(get_color(COLOR_BACKGROUND)))
        .insert_resource(GlobalTextureHandle(None))
        .insert_resource(Settings::default())
        // 커스텀 로직 플러그인 추가
        .add_plugins(BoidPlugin)     // Boid(생명체) 행동 로직
        .add_plugins(GuiPlugin)      // UI 관련
        .add_plugins(StatsPlugin)    // 통계 및 그래프
        .add_plugins(WorldPlugin)    // 월드 초기화 및 배경
        .add_plugins(ElementsPlugin) // 먹이, 독 등 요소 관리
        // 초기 로딩 상태에서 시스템 실행
        .add_systems(OnEnter(SimState::Loading), setup)
        // 업데이트 루프 시스템들
        .add_systems(
            Update,
            camera_clamp_system.run_if(in_state(SimState::Simulating)),
        )
        .add_systems(Update, handle_keyboard_input)
        .add_systems(Update, apply_sim_speed)
        .add_systems(Update, close_on_esc) // ESC 키로 종료
        .run();
}

/// 시뮬레이션 속도를 조절하는 시스템 (현재는 각 시스템 내부에서 sim_speed를 수동으로 처리)
fn apply_sim_speed(_settings: Res<Settings>, _time: ResMut<Time>) {
    // Bevy 0.12에서는 set_relative_speed가 없으므로 시스템별로 개별 처리함
}

/// 초기 자산(스프라이트 시트) 로드 및 카메라 스폰
fn setup(
    mut commands: Commands,
    mut handle: ResMut<GlobalTextureHandle>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut next_state: ResMut<NextState<SimState>>,
) {
    // 스프라이트 시트 로드 및 아틀라스 생성
    let texture_handle = asset_server.load(SPRITE_SHEET_PATH);
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        vec2(TILE_W, TILE_H),
        SPRITE_SHEET_ROWS,
        SPRITE_SHEET_COLS,
        None,
        None,
    );
    handle.0 = Some(texture_atlases.add(texture_atlas));

    // 2D 카메라 스폰 및 PanCam(마우스 조작) 기능 추가
    commands
        .spawn(Camera2dBundle::default())
        .insert(PanCam::default());
        
    // 로딩 완료 후 초기화 상태로 전환
    next_state.set(SimState::InitSim);
}

/// 설정에 따라 카메라가 월드 중앙으로 서서히 돌아오도록 하는 시스템
fn camera_clamp_system(
    settings: Res<Settings>,
    mut cam_query: Query<&mut Transform, With<Camera>>,
) {
    if cam_query.is_empty() {
        return;
    }
    if !settings.camera_clamp_center {
        return;
    }

    let mut cam_transform = cam_query.single_mut();
    // 선형 보간(lerp)을 사용하여 부드럽게 중앙(0,0,0)으로 이동
    cam_transform.translation = cam_transform.translation.lerp(Vec3::ZERO, 0.05);
}

/// 단축키 입력 처리 (카메라 모드 전환, 기즈모 토글, 차트 표시 등)
fn handle_keyboard_input(keys: Res<Input<KeyCode>>, mut settings: ResMut<Settings>) {
    // 1: Boid 추적 모드
    if keys.just_pressed(KeyCode::Key1) {
        settings.camera_follow_boid = !settings.camera_follow_boid;
        settings.camera_follow_predator = false;
        settings.camera_clamp_center = false;
    }
    // 2: 포식자 추적 모드
    if keys.just_pressed(KeyCode::Key2) {
        settings.camera_follow_predator = !settings.camera_follow_predator;
        settings.camera_follow_boid = false;
        settings.camera_clamp_center = false;
    }
    // 3: 중앙 고정 모드
    if keys.just_pressed(KeyCode::Key3) {
        settings.camera_clamp_center = !settings.camera_clamp_center;
        settings.camera_follow_boid = false;
        settings.camera_follow_predator = false;
    }
    // Tab: 시각적 보조(기즈모) 표시 토글
    if keys.just_pressed(KeyCode::Tab) {
        settings.enable_gizmos = !settings.enable_gizmos;
    }
    // Backspace: 통계 그래프 표시 토글
    if keys.just_pressed(KeyCode::Back) {
        settings.show_plots = !settings.show_plots;
    }
    // ` (Grave): 그래프 설정 메뉴 토글
    if keys.just_pressed(KeyCode::Grave) {
        settings.show_plot_settings = !settings.show_plot_settings;
    }
}
