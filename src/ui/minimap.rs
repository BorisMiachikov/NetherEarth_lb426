use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    app::state::AppState,
    core::Team,
    map::grid::{MapGrid, CELL_SIZE},
    player::components::PlayerScout,
    robot::components::RobotMarker,
    structure::{factory::Factory, warbase::Warbase},
};

const MAP_PX: f32 = 160.0; // размер миникарты в пикселях

/// Рисует миникарту в правом нижнем углу.
pub fn draw_minimap(
    state: Res<State<AppState>>,
    mut contexts: EguiContexts,
    map: Res<MapGrid>,
    scouts: Query<&Transform, With<PlayerScout>>,
    robots: Query<(&Transform, &Team), With<RobotMarker>>,
    factories: Query<(&Transform, &Team), (With<Factory>, Without<RobotMarker>)>,
    warbases: Query<(&Transform, &Team), (With<Warbase>, Without<RobotMarker>)>,
) -> Result {
    // Миникарта видна только во время игры и паузы
    if !matches!(
        state.get(),
        AppState::Playing | AppState::Paused
    ) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let map_w = map.width as f32;
    let map_h = map.height as f32;
    let scale_x = MAP_PX / map_w;
    let scale_z = MAP_PX / map_h;

    egui::Window::new("Карта")
        .id(egui::Id::new("minimap"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(true)
        .show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(egui::vec2(MAP_PX, MAP_PX), egui::Sense::hover());
            let rect = response.rect;

            // Фон
            painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(10, 15, 20));

            // --- Структуры ---
            // Фабрики
            for (tf, team) in &factories {
                let px = rect.min.x + (tf.translation.x / CELL_SIZE) * scale_x;
                let py = rect.min.y + (tf.translation.z / CELL_SIZE) * scale_z;
                let color = team_color(*team);
                let half = 2.5;
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        egui::pos2(px, py),
                        egui::vec2(half * 2.0, half * 2.0),
                    ),
                    0.0,
                    color,
                );
            }

            // Варбейсы — крупнее
            for (tf, team) in &warbases {
                let px = rect.min.x + (tf.translation.x / CELL_SIZE) * scale_x;
                let py = rect.min.y + (tf.translation.z / CELL_SIZE) * scale_z;
                let color = team_color(*team);
                let half = 4.0;
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        egui::pos2(px, py),
                        egui::vec2(half * 2.0, half * 2.0),
                    ),
                    1.0,
                    color,
                );
                // Обводка для варбейса
                painter.rect_stroke(
                    egui::Rect::from_center_size(
                        egui::pos2(px, py),
                        egui::vec2(half * 2.0 + 1.0, half * 2.0 + 1.0),
                    ),
                    1.0,
                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                    egui::StrokeKind::Middle,
                );
            }

            // --- Роботы ---
            for (tf, team) in &robots {
                let px = rect.min.x + (tf.translation.x / CELL_SIZE) * scale_x;
                let py = rect.min.y + (tf.translation.z / CELL_SIZE) * scale_z;
                painter.circle_filled(egui::pos2(px, py), 1.8, team_color(*team));
            }

            // --- Скаут (белый) ---
            if let Ok(scout_tf) = scouts.single() {
                let px = rect.min.x + (scout_tf.translation.x / CELL_SIZE) * scale_x;
                let py = rect.min.y + (scout_tf.translation.z / CELL_SIZE) * scale_z;
                // Крестик
                let r = 3.0;
                painter.line_segment(
                    [egui::pos2(px - r, py), egui::pos2(px + r, py)],
                    egui::Stroke::new(1.5, egui::Color32::WHITE),
                );
                painter.line_segment(
                    [egui::pos2(px, py - r), egui::pos2(px, py + r)],
                    egui::Stroke::new(1.5, egui::Color32::WHITE),
                );
                painter.circle_filled(egui::pos2(px, py), 1.5, egui::Color32::WHITE);
            }

            // Рамка
            painter.rect_stroke(
                rect,
                2.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 80, 100)),
                egui::StrokeKind::Middle,
            );
        });

    Ok(())
}

fn team_color(team: Team) -> egui::Color32 {
    match team {
        Team::Player => egui::Color32::from_rgb(60, 140, 255),
        Team::Enemy => egui::Color32::from_rgb(220, 60, 60),
        Team::Neutral => egui::Color32::from_rgb(140, 140, 140),
    }
}
