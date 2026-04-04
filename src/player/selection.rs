use bevy::prelude::*;

use crate::robot::components::RobotMarker;

/// Маркер: робот выбран игроком.
#[derive(Component)]
pub struct Selected;

/// Группа выбора (Ctrl+0..9).
#[derive(Component)]
pub struct SelectionGroup(pub u8);

/// Ресурс: список выбранных сущностей.
#[derive(Resource, Default)]
pub struct SelectionState {
    pub selected: Vec<Entity>,
}

/// ЛКМ на меш робота → выбрать (Shift → добавить).
pub fn on_robot_click(
    click: On<Pointer<Click>>,
    robot_query: Query<Entity, With<RobotMarker>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut selection: ResMut<SelectionState>,
    selected_query: Query<Entity, With<Selected>>,
) {
    let entity = click.entity;
    if click.button != PointerButton::Primary {
        return;
    }
    if robot_query.get(entity).is_err() {
        return;
    }

    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if !shift {
        // Сбросить предыдущий выбор
        for e in &selected_query {
            commands.entity(e).remove::<Selected>();
        }
        selection.selected.clear();
    }

    if selection.selected.contains(&entity) {
        commands.entity(entity).remove::<Selected>();
        selection.selected.retain(|&e| e != entity);
    } else {
        commands.entity(entity).insert(Selected);
        selection.selected.push(entity);
    }
}

/// Ctrl+1..9 — сохранить группу; 1..9 без Ctrl — вызвать группу.
pub fn handle_selection_groups(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut selection: ResMut<SelectionState>,
    selected_query: Query<Entity, With<Selected>>,
    grouped_query: Query<(Entity, &SelectionGroup)>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    let num_keys = [
        (KeyCode::Digit1, 1u8),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
        (KeyCode::Digit4, 4),
        (KeyCode::Digit5, 5),
        (KeyCode::Digit6, 6),
        (KeyCode::Digit7, 7),
        (KeyCode::Digit8, 8),
        (KeyCode::Digit9, 9),
    ];

    for (key, num) in num_keys {
        if !keys.just_pressed(key) {
            continue;
        }
        if ctrl {
            // Сохранить текущий выбор в группу
            for e in &selected_query {
                commands.entity(e).insert(SelectionGroup(num));
            }
        } else {
            // Выбрать всех роботов из группы
            for e in &selected_query {
                commands.entity(e).remove::<Selected>();
            }
            selection.selected.clear();

            for (e, g) in &grouped_query {
                if g.0 == num {
                    commands.entity(e).insert(Selected);
                    selection.selected.push(e);
                }
            }
        }
    }
}

/// Рисует кольцо под выбранными роботами через Gizmos.
pub fn draw_selection_indicators(
    mut gizmos: Gizmos,
    selected: Query<&Transform, (With<Selected>, With<RobotMarker>)>,
) {
    for tf in &selected {
        let pos = tf.translation.with_y(0.05);
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            0.8,
            Color::srgb(1.0, 1.0, 0.0),
        );
    }
}
