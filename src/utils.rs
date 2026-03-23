use std::f32::consts::PI;

use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use bevy_egui::egui::Color32;
use rand::Rng;

use crate::*;

/// 최대 크기가 제한된 벡터 (그래프 데이터 저장 등에 사용)
pub struct LimitedVec<T> {
    pub items: Vec<T>,
    pub max_size: usize,
}

/// 무작위 단위 3D 벡터 반환 (Z축은 0)
pub fn get_rand_unit_vec3() -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(-100.0..100.0);
    let y = rng.gen_range(-100.0..100.0);

    vec3(x, y, 0.0).normalize()
}

/// 무작위 단위 2D 벡터 반환
pub fn get_rand_unit_vec2() -> Vec2 {
    let rand_vec3 = get_rand_unit_vec3();
    vec2(rand_vec3.x, rand_vec3.y)
}

/// 두 위치 사이의 회전 각도 계산
pub fn calc_rotation_angle(v1: Vec3, v2: Vec3) -> f32 {
    let dx = v1.x - v2.x;
    let dy = v1.y - v2.y;

    let mut angle_rad = dy.atan2(dx);

    if angle_rad < 0.0 {
        angle_rad += 2.0 * PI;
    }
    angle_rad
}

/// 월드의 경계 좌표 (min_x, min_y, max_x, max_y) 반환
pub fn get_world_bounds() -> (f32, f32, f32, f32) {
    let min_x = -WORLD_W;
    let min_y = -WORLD_H;
    let max_x = WORLD_W;
    let max_y = WORLD_H;

    (min_x, min_y, max_x, max_y)
}

/// 지정된 좌표가 월드 경계 내에 있도록 제한
pub fn limit_to_world((x, y): (f32, f32)) -> (f32, f32) {
    let (x, y) = (x.min(WORLD_W), y.min(WORLD_H));
    let (x, y) = (x.max(-WORLD_W), y.max(-WORLD_H));
    (x, y)
}

/// 목표 지점으로 가기 위한 조향 힘(Steering Force) 계산
pub fn get_steering_force(target: Vec2, pos: Vec2, velocity: Vec2) -> Vec2 {
    let desired = target - pos;
    desired - velocity
}

/// (u8, u8, u8) 튜플을 Bevy Color로 변환
pub fn get_color((r, g, b): (u8, u8, u8)) -> Color {
    Color::rgb_u8(r, g, b)
}

/// Hex 문자열을 Bevy Color로 변환
pub fn get_color_hex(value: &str) -> Color {
    Color::hex(value).unwrap()
}

/// (u8, u8, u8) 튜플을 egui Color32로 변환
pub fn get_color32((r, g, b): (u8, u8, u8)) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Hex 문자열을 RGBA 튜플로 변환
pub fn hex_to_rgba(hex: &str) -> Option<(u8, u8, u8, u8)> {
    let alpha = match hex.len() {
        6 => false,
        8 => true,
        _ => None?,
    };
    u32::from_str_radix(hex, 16)
        .ok()
        .map(|u| if alpha { u } else { u << 8 | 0xff })
        .map(u32::to_be_bytes)
        .map(|[r, g, b, a]| (r, g, b, a))
}

impl<T> LimitedVec<T> {
    pub fn new() -> Self {
        LimitedVec {
            items: Vec::new(),
            max_size: MAX_NUM_POINTS,
        }
    }

    /// 데이터를 추가하되, 최대 크기를 넘으면 가장 오래된 데이터 제거
    pub fn push(&mut self, item: T) {
        self.items.push(item);
        if self.items.len() > self.max_size {
            self.items.remove(0);
        }
    }
}

