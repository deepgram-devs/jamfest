use super::SpeechEvent;
use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(keyboard_input);
    }
}

/// Shortcut keystrokes in case Deepgram isn't working or or is flaky or the individual cannot
/// otherwise use the speech feature easily. Also good for debugging.
fn keyboard_input(keys: Res<Input<KeyCode>>, mut speech_events: EventWriter<SpeechEvent>) {
    if keys.just_pressed(KeyCode::J) {
        info!("Sending sugar speech event triggered by key press");
        speech_events.send(SpeechEvent::Sugar);
    } else if keys.just_pressed(KeyCode::B) {
        info!("Sending bridge speech event triggered by key press");
        speech_events.send(SpeechEvent::Bridge);
    } else if keys.just_pressed(KeyCode::M) {
        info!("Sending mentos speech event triggered by key press");
        speech_events.send(SpeechEvent::Mentos);
    }
}
