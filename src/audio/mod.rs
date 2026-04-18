use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

use crate::{
    app::state::AppState,
    core::events::{EntityDamaged, EntityDestroyed, StructureCaptured},
    player::Selected,
};

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

#[derive(Resource)]
struct SoundHandles {
    shot: Handle<AudioSource>,
    explosion: Handle<AudioSource>,
    construction: Handle<AudioSource>,
    select: Handle<AudioSource>,
    music: Handle<AudioSource>,
}

#[derive(Component)]
struct MusicPlayer;

fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SoundHandles {
        shot:         asset_server.load("audio/sfx/shot.wav"),
        explosion:    asset_server.load("audio/sfx/explosion.wav"),
        construction: asset_server.load("audio/sfx/construction.wav"),
        select:       asset_server.load("audio/sfx/select.wav"),
        music:        asset_server.load("audio/music/mammoth.ogg"),
    });
}

fn start_music(
    mut commands: Commands,
    sounds: Res<SoundHandles>,
    settings: Res<AudioSettings>,
) {
    commands.spawn((
        AudioPlayer::new(sounds.music.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::Linear(settings.music_volume),
            ..default()
        },
        MusicPlayer,
    ));
}

fn stop_music(mut commands: Commands, music: Query<Entity, With<MusicPlayer>>) {
    for e in &music {
        commands.entity(e).despawn();
    }
}

fn play_sfx(commands: &mut Commands, handle: Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioPlayer::new(handle),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(volume),
            ..default()
        },
    ));
}

fn on_shot(
    trigger: On<EntityDamaged>,
    sounds: Res<SoundHandles>,
    settings: Res<AudioSettings>,
    mut commands: Commands,
) {
    if trigger.event().attacker.is_some() {
        play_sfx(&mut commands, sounds.shot.clone(), settings.sfx_volume);
    }
}

fn on_destroyed(
    _trigger: On<EntityDestroyed>,
    sounds: Res<SoundHandles>,
    settings: Res<AudioSettings>,
    mut commands: Commands,
) {
    play_sfx(&mut commands, sounds.explosion.clone(), settings.sfx_volume);
}

fn on_captured(
    _trigger: On<StructureCaptured>,
    sounds: Res<SoundHandles>,
    settings: Res<AudioSettings>,
    mut commands: Commands,
) {
    play_sfx(&mut commands, sounds.construction.clone(), settings.sfx_volume);
}

fn on_unit_selected(
    added: Query<Entity, Added<Selected>>,
    sounds: Res<SoundHandles>,
    settings: Res<AudioSettings>,
    mut commands: Commands,
) {
    if !added.is_empty() {
        play_sfx(&mut commands, sounds.select.clone(), settings.sfx_volume);
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioSettings>()
            .add_systems(Startup, load_sounds)
            .add_systems(OnEnter(AppState::Playing), start_music)
            .add_systems(OnExit(AppState::Playing), stop_music)
            .add_systems(
                Update,
                on_unit_selected.run_if(in_state(AppState::Playing)),
            )
            .add_observer(on_shot)
            .add_observer(on_destroyed)
            .add_observer(on_captured);
    }
}
