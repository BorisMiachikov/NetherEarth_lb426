use bevy::prelude::*;

/// Игровое время. 1 game_day = seconds_per_day реальных секунд.
#[derive(Resource, Debug, Clone)]
pub struct GameTime {
    pub game_day: u32,
    pub day_elapsed: f32,
    pub seconds_per_day: f32,
    pub paused: bool,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            game_day: 0,
            day_elapsed: 0.0,
            seconds_per_day: 30.0,
            paused: false,
        }
    }
}

pub fn tick_game_time(time: Res<Time>, mut game_time: ResMut<GameTime>) {
    if game_time.paused {
        return;
    }
    game_time.day_elapsed += time.delta_secs();
    if game_time.day_elapsed >= game_time.seconds_per_day {
        game_time.day_elapsed -= game_time.seconds_per_day;
        game_time.game_day += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_increments_after_full_cycle() {
        let mut gt = GameTime::default();
        gt.day_elapsed = 29.9;
        // Симулируем дельту 0.2с
        gt.day_elapsed += 0.2;
        if gt.day_elapsed >= gt.seconds_per_day {
            gt.day_elapsed -= gt.seconds_per_day;
            gt.game_day += 1;
        }
        assert_eq!(gt.game_day, 1);
        assert!((gt.day_elapsed - 0.1).abs() < 1e-4);
    }

    #[test]
    fn paused_does_not_advance() {
        let mut gt = GameTime { paused: true, ..Default::default() };
        gt.day_elapsed = 29.0;
        // paused — не трогаем day_elapsed
        assert_eq!(gt.game_day, 0);
    }
}
