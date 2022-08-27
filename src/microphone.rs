use super::SpeechEvent;
use bevy::prelude::*;

use fon::{mono::Mono32, Audio, Frame};
use pasts::exec;
use wavy::{Microphone, MicrophoneStream};

pub struct MicrophonePlugin;
impl Plugin for MicrophonePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MicrophoneReceiver>()
            .add_startup_system(setup_deepgram.exclusive_system())
            .add_system(handle_asr);
    }
}

/// We will have one handle for the microphone as a global resource.
struct MicrophoneReceiver {
    rx: crossbeam_channel::Receiver<Vec<i16>>,
}

impl FromWorld for MicrophoneReceiver {
    fn from_world(_world: &mut World) -> Self {
        let (audio_sender, audio_receiver) = crossbeam_channel::unbounded();

        connect_to_microphone(audio_sender);

        info!("Connected to microphone.");

        MicrophoneReceiver { rx: audio_receiver }
    }
}

/// We are using a non-send resource to handle the websocket client.
/// See more here: https://bevy-cheatbook.github.io/programming/non-send.html
/// We are also temporarily using a proxy websocket server to handle credentials.
fn setup_deepgram(world: &mut World) {
    let client = wasm_sockets::PollingClient::new("wss://spookyspeechspells.deepgram.com/deepgram_websockets_proxy?encoding=linear16&sample_rate=44100&channels=1").unwrap();

    info!("Connected to Deepgram. Probably.");

    world.insert_non_send_resource(DeepgramWebsocket {
        client: Some(client),
    });
}

/// This will be a non-send resource, which is perfect for polling clients
/// which poll in a bevy system which occurs once per frame ish.
#[derive(Default)]
struct DeepgramWebsocket {
    client: Option<wasm_sockets::PollingClient>,
}

/// This is based on the following example: https://github.com/libcala/wavy/blob/stable/examples/record/src/main.rs
fn connect_to_microphone(tx: crossbeam_channel::Sender<Vec<i16>>) {
    let mut state = State {
        buffer: Audio::with_silence(44_100, 0),
        tx,
    };
    let mut microphone = Microphone::default();

    exec!(state.event(pasts::wait! {
        Event::Record(microphone.record().await),
    }))
}

/// An event handled by some microphone event loop.
enum Event<'a> {
    /// This event occurs when the microphone has recorded some audio.
    Record(MicrophoneStream<'a, Mono32>),
}

/// A state for handling the microphone audio stream.
struct State {
    /// Temporary buffer for holding real-time audio samples.
    buffer: Audio<Mono32>,
    /// The sending half of a channel, used to send the audio to another system.
    tx: crossbeam_channel::Sender<Vec<i16>>,
}

impl State {
    /// Some microphone event loop.
    fn event(&mut self, event: Event<'_>) {
        match event {
            // if we got an event of new audio recorded by the microphone,
            // convert the audio to i16 pcm and send it along via a channel
            Event::Record(microphone_stream) => {
                // TODO: hopefully there is another way, outside of a loop, to
                // determine the sample rate. Empirically, I found it was 44100 Hz,
                // but this may vary by system and browser.
                //info!("Sample rate: {:?}.", microphone_stream.sample_rate());

                let mut audio_buffer = Vec::new();
                self.buffer.extend(microphone_stream);

                for frame in self.buffer.drain() {
                    let sample: f32 = frame.channels()[0].into();
                    audio_buffer.push(f32_to_i16(sample));
                }

                let _ = self.tx.send(audio_buffer.to_owned());
            }
        }
    }
}

/// A helper function for converting f32 PCM samples to i16 (linear16) samples.
/// Deepgram currently does not support f32 PCM.
fn f32_to_i16(sample: f32) -> i16 {
    let sample = sample * 32768.0;

    // This is a saturating cast. For more details, see:
    // <https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast>.
    sample as i16
}

/// A helper function for converting a vector of i16 samples to Vec<u8>
/// in order to pass on to our websocket client.
pub fn to_vec_u8(input: Vec<i16>) -> Vec<u8> {
    let mut vec_u8 = Vec::with_capacity(2 * input.len());

    for value in input {
        vec_u8.extend(&value.to_le_bytes());
    }

    vec_u8
}

fn handle_asr(
    microphone_receiver: Res<MicrophoneReceiver>,
    mut deepgram_websocket: NonSendMut<DeepgramWebsocket>,
    mut speech_events: EventWriter<SpeechEvent>,
) {
    if let Some(client) = &mut deepgram_websocket.client {
        while let Ok(audio_buffer) = microphone_receiver.rx.try_recv() {
            client.send_binary(to_vec_u8(audio_buffer)).unwrap();
        }

        for message in client.receive() {
            if let wasm_sockets::Message::Text(message) = message {
                info!("Received a message from Deepgram: {:?}", message);
                if message.contains("sugar") {
                    info!("Sending sugar speech event.");
                    speech_events.send(SpeechEvent::Sugar);
                }
                if message.contains("mentos") {
                    info!("Sending mentos speech event.");
                    speech_events.send(SpeechEvent::Mentos);
                }
                if message.contains("bridge") {
                    info!("Sending bridge speech event.");
                    speech_events.send(SpeechEvent::Bridge);
                }
            }
        }
    }
}
