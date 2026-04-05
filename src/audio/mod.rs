use bevy::prelude::*;

/// Настройки громкости (загружаются из configs/audio.ron в фазе 8.10).
#[derive(Resource, Debug, Clone)]
pub struct AudioSettings {
    pub sfx_volume: f32,
    pub music_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sfx_volume: 0.8,
            music_volume: 0.4,
        }
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioSettings>();
        // Фаза 8.10: воспроизведение SFX добавляется после поставки assets/audio/*.ogg
    }
}
