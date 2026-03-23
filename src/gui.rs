use bevy::{math::vec2, prelude::*};
use bevy_egui::{
    egui::{self, epaint, Color32, Ui},
    EguiContexts, EguiPlugin, EguiSettings,
};
use egui_plot::{Line, Plot, PlotPoints, PlotResponse};

use crate::boid::{Boid, Dna, Predator, PredatorDna};
use crate::stats::*;
use crate::utils::{get_color, get_color32};
use crate::*;

/// GUI 및 디버그 시각화를 관리하는 플러그인
pub struct GuiPlugin;

/// 그래프에 표시할 데이터 구조체
struct PlotData<'a> {
    points: &'a Vec<f32>,
    color: Color32,
}

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(OnEnter(SimState::InitSim), setup_egui)
            .add_systems(
                Update,
                (
                    update_egui_plots,          // 통계 그래프 업데이트
                    draw_boid_debug_gizmos,     // Boid 인식 범위 표시
                    draw_predator_debug_gizmos, // 포식자 인식 범위 표시
                    draw_world_boundary,        // 월드 경계선 표시
                    show_plot_settings,         // 설정 창 표시
                )
                    .run_if(in_state(SimState::Simulating)),
            );
    }
}

/// egui 초기 설정 (UI 배율 등)
fn setup_egui(mut egui_settings: ResMut<EguiSettings>) {
    egui_settings.scale_factor = 1.5;
}

/// 월드의 경계를 사각형 기즈모로 그림
fn draw_world_boundary(mut gizmos: Gizmos) {
    gizmos.rect_2d(
        Vec2::ZERO,
        0.0,
        vec2(WORLD_W * 2.0, WORLD_H * 2.0),
        Color::GRAY,
    );
}

/// 일반 Boid들의 각종 인식 범위를 원형 기즈모로 그림
fn draw_boid_debug_gizmos(
    mut gizmos: Gizmos,
    settings: Res<Settings>,
    boids_query: Query<(&Transform, &Dna), (With<Boid>, Without<Predator>)>,
) {
    if !settings.enable_gizmos || boids_query.is_empty() {
        return;
    }

    for (transform, dna) in boids_query.iter() {
        // 초록: 먹이 인식 범위
        gizmos.circle_2d(
            transform.translation.truncate(),
            dna.food_perception_radius,
            Color::GREEN,
        );
        // 빨강: 독 인식 범위
        gizmos.circle_2d(
            transform.translation.truncate(),
            dna.poison_perception_radius,
            Color::RED,
        );
        // 연노랑: 포식자 인식 범위
        gizmos.circle_2d(
            transform.translation.truncate(),
            dna.predator_perception_radius,
            get_color(COLOR_PREDATOR),
        );
    }
}

/// 포식자의 사냥감 인식 범위를 원형 기즈모로 그림
fn draw_predator_debug_gizmos(
    mut gizmos: Gizmos,
    settings: Res<Settings>,
    predator_query: Query<(&Transform, &PredatorDna), With<Predator>>,
) {
    if !settings.enable_gizmos || predator_query.is_empty() {
        return;
    }

    for (transform, predator_dna) in predator_query.iter() {
        // 하늘색: 사냥감 인식 범위
        gizmos.circle_2d(
            transform.translation.truncate(),
            predator_dna.prey_perception,
            Color::CYAN,
        );
    }
}

/// 시뮬레이션 속도 및 그래프 표시 옵션을 조절하는 설정 창
fn show_plot_settings(mut contexts: EguiContexts, mut settings: ResMut<Settings>) {
    if !settings.show_plot_settings {
        return;
    }

    egui::Window::new("Settings").show(contexts.ctx_mut(), |ui| {
        egui::CollapsingHeader::new("Simulation")
            .default_open(true)
            .show(ui, |ui| {
                ui.add(egui::Slider::new(&mut settings.sim_speed, 1.0..=100.0).text("Speed multiplier"));
            });

        egui::CollapsingHeader::new("Graphs")
            .default_open(true)
            .show(ui, |ui| {
                ui.checkbox(&mut settings.plot_options.num_boids, "Number of Boids");
                ui.checkbox(&mut settings.plot_options.lifespan, "Lifespan");
                ui.checkbox(&mut settings.plot_options.perception, "Perception");
                ui.checkbox(&mut settings.plot_options.affinity, "Affinity");
                ui.checkbox(&mut settings.plot_options.steering, "Steering");
                ui.checkbox(&mut settings.plot_options.speed, "Speed");
            });
    });
}

/// 우측 상단에 실시간 데이터 통계 그래프를 렌더링
fn update_egui_plots(
    mut contexts: EguiContexts,
    stats: Res<SimulationStats>,
    settings: Res<Settings>,
) {
    if !settings.show_plots {
        return;
    }

    let ctx = contexts.ctx_mut();
    let old = ctx.style().visuals.clone();
    // 투명한 배경의 어두운 테마 설정
    ctx.set_visuals(egui::Visuals {
        window_fill: Color32::from_rgba_premultiplied(0, 0, 0, 130),
        panel_fill: Color32::from_rgba_premultiplied(0, 0, 0, 130),
        window_stroke: egui::Stroke {
            color: Color32::TRANSPARENT,
            width: 0.0,
        },
        window_shadow: epaint::Shadow {
            color: Color32::TRANSPARENT,
            ..old.window_shadow
        },
        ..old
    });

    egui::Window::new("")
        .title_bar(false)
        .default_pos(egui::pos2(1500.0, 0.0))
        .default_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            // 개체 수 통계
            if settings.plot_options.num_boids {
                get_plot(
                    "Number of Boids",
                    vec![
                        PlotData {
                            color: Color32::RED,
                            points: &stats.num_poison.items,
                        },
                        PlotData {
                            color: Color32::GREEN,
                            points: &stats.num_food.items,
                        },
                        PlotData {
                            color: get_color32(COLOR_PREDATOR_LOW_HEALTH),
                            points: &stats.num_predators.items,
                        },
                        PlotData {
                            color: Color32::WHITE,
                            points: &stats.num_boids.items,
                        },
                    ],
                    ui,
                );
            }
            // 평균 수명 통계
            if settings.plot_options.lifespan {
                get_plot(
                    "Lifespan",
                    vec![
                        PlotData {
                            color: Color32::WHITE,
                            points: &stats.avg_lifespan.items,
                        },
                        PlotData {
                            color: get_color32(COLOR_PREDATOR_LOW_HEALTH),
                            points: &stats.avg_predator_lifespan.items,
                        },
                    ],
                    ui,
                );
            }
            // 인식 범위 통계 (DNA 평균값)
            if settings.plot_options.perception {
                get_plot(
                    "Perception",
                    vec![
                        PlotData {
                            color: Color32::GREEN,
                            points: &stats.food_perception.items,
                        },
                        PlotData {
                            color: Color32::RED,
                            points: &stats.poison_perception.items,
                        },
                        PlotData {
                            color: get_color32(COLOR_PREDATOR),
                            points: &stats.predator_perception.items,
                        },
                        PlotData {
                            color: get_color32(COLOR_BOID_LOW_HEALTH),
                            points: &stats.prey_perception.items,
                        },
                    ],
                    ui,
                );
            }
            // 먹이/독 서호도 통계
            if settings.plot_options.affinity {
                get_plot(
                    "Affinity",
                    vec![
                        PlotData {
                            color: Color32::GREEN,
                            points: &stats.food_affinity.items,
                        },
                        PlotData {
                            color: Color32::RED,
                            points: &stats.poison_affinity.items,
                        },
                        PlotData {
                            color: get_color32(COLOR_BOID_LOW_HEALTH),
                            points: &stats.prey_affinity.items,
                        },
                        PlotData {
                            color: get_color32(COLOR_PREDATOR_LOW_HEALTH),
                            points: &stats.predator_affinity.items,
                        },
                    ],
                    ui,
                );
            }
            // 조향 힘 통계
            if settings.plot_options.steering {
                get_plot(
                    "Steering Force",
                    vec![PlotData {
                        color: Color32::WHITE,
                        points: &stats.steering_force.items,
                    }],
                    ui,
                );
            }
            // 이동 속도 통계
            if settings.plot_options.speed {
                get_plot(
                    "Speed",
                    vec![
                        PlotData {
                            color: Color32::WHITE,
                            points: &stats.speed.items,
                        },
                        PlotData {
                            color: get_color32(COLOR_PREDATOR_LOW_HEALTH),
                            points: &stats.predator_speed.items,
                        },
                    ],
                    ui,
                );
            }
        });
}

/// 개별 접이식 차트 영역 생성
fn get_plot(
    title: &str,
    plots_data: Vec<PlotData>,
    ui: &mut Ui,
) -> egui::CollapsingResponse<PlotResponse<()>> {
    let mut lines = Vec::new();

    for plot in plots_data {
        let curve: PlotPoints = (0..plot.points.len())
            .map(|i| [i as f64, plot.points[i] as f64])
            .collect();
        let line = Line::new(curve).width(PLOT_LINE_WIDTH).color(plot.color);
        lines.push(line);
    }

    egui::CollapsingHeader::new(title)
        .default_open(true)
        .show_unindented(ui, |ui| {
            Plot::new(title)
                .show_axes(false)
                .show_background(false)
                .show_grid(false)
                .view_aspect(2.0)
                .auto_bounds_x()
                .auto_bounds_y()
                .show(ui, |plot_ui| {
                    for l in lines {
                        plot_ui.line(l)
                    }
                })
        })
}

