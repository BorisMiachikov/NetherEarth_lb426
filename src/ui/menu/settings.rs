use bevy::{prelude::*, window::WindowResolution};
use bevy_egui::egui;

use crate::{audio::AudioSettings, localization::{ChangeLanguage, Language, Localization}};

const RESOLUTIONS: &[(u32, u32)] = &[(1280, 720), (1600, 900), (1920, 1080)];

/// Отрисовка панели настроек. Возвращает true если нажато «Назад».
pub fn draw_settings(
    ui: &mut egui::Ui,
    loc: &Localization,
    audio: &mut AudioSettings,
    window: Option<&mut Window>,
    commands: &mut Commands,
) -> bool {
    let mut go_back = false;

    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(loc.t("settings.title"))
                .size(18.0)
                .strong()
                .color(egui::Color32::from_rgb(200, 200, 200)),
        );
        ui.add_space(10.0);
    });

    ui.separator();

    // — Звук —
    ui.add_space(6.0);
    ui.label(egui::RichText::new(loc.t("settings.audio")).strong().color(egui::Color32::from_rgb(180, 180, 80)));
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(loc.t("settings.music")).color(egui::Color32::GRAY).size(13.0));
        ui.add(egui::Slider::new(&mut audio.music_volume, 0.0..=1.0).show_value(false));
        ui.label(format!("{:.0}%", audio.music_volume * 100.0));
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(loc.t("settings.sfx")).color(egui::Color32::GRAY).size(13.0));
        ui.add(egui::Slider::new(&mut audio.sfx_volume, 0.0..=1.0).show_value(false));
        ui.label(format!("{:.0}%", audio.sfx_volume * 100.0));
    });

    ui.add_space(8.0);
    ui.separator();

    // — Экран —
    ui.add_space(6.0);
    ui.label(egui::RichText::new(loc.t("settings.display")).strong().color(egui::Color32::from_rgb(180, 180, 80)));
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(loc.t("settings.language")).color(egui::Color32::GRAY).size(13.0));
        ui.add_space(4.0);
        for lang in [Language::Russian, Language::English] {
            let active = loc.language == lang;
            let btn = egui::Button::new(
                egui::RichText::new(lang.label())
                    .color(if active { egui::Color32::from_rgb(80, 190, 255) } else { egui::Color32::GRAY }),
            );
            if ui.add_enabled(!active, btn).clicked() {
                commands.trigger(ChangeLanguage(lang));
            }
        }
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(loc.t("settings.resolution")).color(egui::Color32::GRAY).size(13.0));
        ui.add_space(4.0);
        if let Some(win) = window {
            let cur_w = win.width() as u32;
            let cur_h = win.height() as u32;
            for &(w, h) in RESOLUTIONS {
                let active = cur_w == w && cur_h == h;
                let label = format!("{}×{}", w, h);
                let btn = egui::Button::new(
                    egui::RichText::new(&label)
                        .color(if active { egui::Color32::from_rgb(80, 190, 255) } else { egui::Color32::GRAY })
                        .size(12.0),
                );
                if ui.add_enabled(!active, btn).clicked() {
                    win.resolution = WindowResolution::new(w, h);
                }
            }
        }
    });

    ui.add_space(8.0);
    ui.separator();

    // — Клавиши —
    ui.add_space(6.0);
    ui.label(egui::RichText::new(loc.t("settings.keybindings")).strong().color(egui::Color32::from_rgb(180, 180, 80)));
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .max_height(180.0)
        .id_salt("settings_keybindings")
        .show(ui, |ui| {
            let key   = egui::Color32::from_rgb(120, 220, 255);
            let desc  = egui::Color32::from_rgb(190, 190, 190);
            let hdr   = egui::Color32::from_rgb(160, 160, 80);

            macro_rules! hdr { ($t:expr) => {
                ui.add_space(4.0);
                ui.label(egui::RichText::new($t).small().strong().color(hdr));
            }; }
            macro_rules! row { ($k:expr, $d:expr) => {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new($k).monospace().strong().color(key).small());
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new($d).color(desc).small());
                });
            }; }

            hdr!(loc.t("help.section.scout"));
            row!("W/A/S/D",       loc.t("help.key.wasd"));
            row!("Q / E",         loc.t("help.key.qe"));

            hdr!("Камера");
            row!("Scroll",        loc.t("help.key.scroll"));
            row!("ССК + drag",    "Orbit");
            row!("Z / C",         "Поворот камеры");

            hdr!(loc.t("help.section.selection"));
            row!("ЛКМ",           loc.t("help.key.lmb"));
            row!("Shift+ЛКМ",     loc.t("help.key.shift_lmb"));
            row!("Ctrl+1-9",      loc.t("help.key.ctrl_num"));
            row!("1-9",           loc.t("help.key.num"));

            hdr!(loc.t("help.section.commands"));
            row!("ПКМ",           loc.t("help.key.rmb"));
            row!("P+ПКМ",         loc.t("help.key.p_rmb"));

            hdr!(loc.t("help.section.build"));
            row!("B",             loc.t("help.key.b"));

            hdr!(loc.t("help.section.other"));
            row!("Esc",           loc.t("help.key.esc"));
            row!("F1",            loc.t("help.key.f1"));
        });

    ui.add_space(10.0);
    ui.vertical_centered(|ui| {
        if ui.add_sized([190.0, 32.0], egui::Button::new(loc.t("menu.btn.back"))).clicked() {
            go_back = true;
        }
    });
    ui.add_space(6.0);

    go_back
}
