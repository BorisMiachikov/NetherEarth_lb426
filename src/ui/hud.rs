use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    core::time::GameTime,
    economy::resource::{PlayerResources, ResourceType},
};

const RESOURCE_LABELS: &[(ResourceType, &str)] = &[
    (ResourceType::General, "Общий"),
    (ResourceType::Chassis, "Шасси"),
    (ResourceType::Cannon, "Пушки"),
    (ResourceType::Missile, "Ракеты"),
    (ResourceType::Phasers, "Фазеры"),
    (ResourceType::Electronics, "Электроника"),
    (ResourceType::Nuclear, "Ядерный"),
];

/// HUD — панель ресурсов игрока (верхний левый угол).
pub fn draw_resource_hud(
    player_res: Res<PlayerResources>,
    game_time: Res<GameTime>,
    mut contexts: EguiContexts,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::Window::new("Ресурсы")
        .id(egui::Id::new("resource_hud"))
        .default_pos([10.0, 10.0])
        .resizable(false)
        .collapsible(false)
        .title_bar(true)
        .show(ctx, |ui| {
            ui.label(format!("День: {}", game_time.game_day));
            ui.separator();

            egui::Grid::new("res_grid")
                .num_columns(2)
                .spacing([16.0, 2.0])
                .show(ui, |ui| {
                    for (rt, label) in RESOURCE_LABELS {
                        ui.label(*label);
                        ui.label(format!("{}", player_res.get(*rt)));
                        ui.end_row();
                    }
                });
        });

    Ok(())
}
