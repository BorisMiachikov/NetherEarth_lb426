use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    core::{time::GameTime, Team},
    economy::resource::{PlayerResources, ResourceType},
    structure::{capture::Capturable, factory::Factory, FactType},
};

/// (тип ресурса, метка, emoji)
const RESOURCE_META: &[(ResourceType, &str, &str)] = &[
    (ResourceType::General,     "Общий",      "⬡"),
    (ResourceType::Chassis,     "Шасси",      "⬜"),
    (ResourceType::Cannon,      "Пушки",      "⦿"),
    (ResourceType::Missile,     "Ракеты",     "↑"),
    (ResourceType::Phasers,     "Фазеры",     "~"),
    (ResourceType::Electronics, "Электроника","⚙"),
    (ResourceType::Nuclear,     "Ядерный",    "☢"),
];

/// Цвет количества ресурса: красный (мало) → жёлтый → зелёный.
fn resource_color(amount: i32) -> egui::Color32 {
    if amount <= 5 {
        egui::Color32::from_rgb(220, 60, 60)
    } else if amount <= 20 {
        egui::Color32::from_rgb(230, 180, 40)
    } else {
        egui::Color32::from_rgb(80, 220, 100)
    }
}

/// Считает суммарное производство в день для каждой команды.
fn production_per_day(
    factories: &Query<(&FactType, &Team), (With<Factory>, With<Capturable>)>,
) -> [i32; 7] {
    // [General, Chassis, Cannon, Missile, Phasers, Electronics, Nuclear] — по порядку RESOURCE_META
    let order = [
        ResourceType::General,
        ResourceType::Chassis,
        ResourceType::Cannon,
        ResourceType::Missile,
        ResourceType::Phasers,
        ResourceType::Electronics,
        ResourceType::Nuclear,
    ];
    let mut prod = [0i32; 7];
    for (ft, team) in factories {
        if *team != Team::Player {
            continue;
        }
        let rt = match ft {
            FactType::General     => ResourceType::General,
            FactType::Chassis     => ResourceType::Chassis,
            FactType::Cannon      => ResourceType::Cannon,
            FactType::Missile     => ResourceType::Missile,
            FactType::Phasers     => ResourceType::Phasers,
            FactType::Electronics => ResourceType::Electronics,
            FactType::Nuclear     => ResourceType::Nuclear,
        };
        // +5 специфического + 2 General за каждую фабрику
        if let Some(idx) = order.iter().position(|r| *r == rt) {
            prod[idx] += 5;
        }
        prod[0] += 2; // General
    }
    prod
}

/// HUD — панель ресурсов игрока (верхний левый угол).
pub fn draw_resource_hud(
    player_res: Res<PlayerResources>,
    game_time: Res<GameTime>,
    factories: Query<(&FactType, &Team), (With<Factory>, With<Capturable>)>,
    enemy_factories: Query<&Team, With<Factory>>,
    mut contexts: EguiContexts,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let prod = production_per_day(&factories);

    // Подсчёт фабрик по командам
    let (player_count, enemy_count, neutral_count) = enemy_factories.iter().fold(
        (0u32, 0u32, 0u32),
        |(p, e, n), team| match team {
            Team::Player  => (p + 1, e, n),
            Team::Enemy   => (p, e + 1, n),
            Team::Neutral => (p, e, n + 1),
        },
    );

    egui::Window::new("Ресурсы")
        .id(egui::Id::new("resource_hud"))
        .default_pos([10.0, 10.0])
        .resizable(false)
        .collapsible(true)
        .show(ctx, |ui| {
            // --- Заголовок: день + счёт фабрик ---
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("День {}", game_time.game_day))
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 200)),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("⬡{player_count}"))
                        .color(egui::Color32::from_rgb(60, 140, 255))
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("⬡{enemy_count}"))
                        .color(egui::Color32::from_rgb(220, 60, 60))
                        .strong(),
                );
                if neutral_count > 0 {
                    ui.label(
                        egui::RichText::new(format!("⬡{neutral_count}"))
                            .color(egui::Color32::GRAY),
                    );
                }
            });

            ui.separator();

            // --- Таблица ресурсов ---
            egui::Grid::new("res_grid")
                .num_columns(3)
                .spacing([8.0, 3.0])
                .show(ui, |ui| {
                    for (i, (rt, label, icon)) in RESOURCE_META.iter().enumerate() {
                        let amount = player_res.get(*rt);
                        let day_prod = prod[i];

                        // Иконка + метка
                        ui.label(
                            egui::RichText::new(format!("{icon} {label}"))
                                .color(egui::Color32::from_rgb(160, 160, 180)),
                        );

                        // Количество
                        ui.label(
                            egui::RichText::new(format!("{amount}"))
                                .color(resource_color(amount))
                                .strong(),
                        );

                        // Производство в день (только если есть)
                        if day_prod > 0 {
                            ui.label(
                                egui::RichText::new(format!("+{day_prod}/д"))
                                    .color(egui::Color32::from_rgb(80, 180, 80))
                                    .small(),
                            );
                        } else {
                            ui.label("");
                        }

                        ui.end_row();
                    }
                });
        });

    Ok(())
}
