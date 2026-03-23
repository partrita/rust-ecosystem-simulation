//! 생태계 시뮬레이션의 핵심 로직을 담고 있는 라이브러리 모듈입니다.

pub mod boid;     // 생명체(Boid) 및 포식자 로직
pub mod configs;  // 시뮬레이션 설정 및 상수
pub mod elements; // 먹이, 독 등 환경 요소
pub mod globals;  // 전역 리소스 및 상태
pub mod gui;      // 사용자 인터페이스 및 그래프
pub mod stats;    // 통계 수집 데이터
pub mod utils;    // 유틸리티 함수
pub mod world;    // 월드 초기화 및 관리

pub use configs::*;
pub use globals::*;
