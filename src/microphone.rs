use bevy::prelude::*;
use futures::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
};
use tokio::runtime::Runtime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpeechEvent {
    Bridge,
    Mentos,
    Sugar,
}

pub struct MicrophonePlugin;
impl Plugin for MicrophonePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AsyncRuntime>()
            .init_resource::<MicrophoneReceiver>()
            .init_resource::<DeepgramWebsocket>()
            .add_system(handle_asr);
    }
}

/// This async function must be executed in an async runtime, and it will return a websocket handle
/// to Deepgram, which can be used to send and receive messages, although sending and receiving must
/// also be executed in an async runtime.
async fn connect_to_deepgram(
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY").expect("Deepgram API Key is required.");

    // prepare the connection request with the api key authentication
    // TODO: don't hardcode the encoding, sample rate, or number of channels
    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri("wss://api.deepgram.com/v1/listen?encoding=linear16&sample_rate=44100&channels=1&tier=enhanced")
        .header("Authorization", format!("Token {}", api_key))
        .body(())
        .expect("Failed to build a connection request to Deepgram.");

    // actually finally connect to deepgram
    // we do this using the prepared http request so that we can get the auth header in there
    let (deepgram_socket, _) = tokio_tungstenite::connect_async(request)
        .await
        .expect("Failed to connect to Deepgram.");

    deepgram_socket
}

/// We will have one handle for the microphone as a global resource.
struct MicrophoneReceiver {
    rx: crossbeam_channel::Receiver<Vec<f32>>,
}

impl FromWorld for MicrophoneReceiver {
    fn from_world(_world: &mut World) -> Self {
        let (audio_sender, audio_receiver) = crossbeam_channel::unbounded();

        connect_to_microphone(audio_sender);

        info!("Connected to microphone.");

        MicrophoneReceiver { rx: audio_receiver }
    }
}

/// We will pass around an Arc'd Tokio Runtime as a global resourse to
/// be used when executing async tasks.
struct AsyncRuntime {
    rt: std::sync::Arc<Runtime>,
}

impl FromWorld for AsyncRuntime {
    fn from_world(_world: &mut World) -> Self {
        AsyncRuntime {
            rt: std::sync::Arc::new(Runtime::new().unwrap()),
        }
    }
}

/// We will have a single handle for a Deepgram websocket connection as a global resource.
/// This DeepgramWebsocket object/resource will contain a `tx` for sending websocket messages
/// to Deepgram, and an `rx` for handling websocket messages received from Deepgram. Note that
/// the `tx` must be used in an async runtime, while the `rx` can be used in any runtime.
struct DeepgramWebsocket {
    tx: SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tungstenite::Message,
    >,
    rx: crossbeam_channel::Receiver<tungstenite::Message>,
}

impl FromWorld for DeepgramWebsocket {
    fn from_world(world: &mut World) -> Self {
        let rt = world.get_resource::<AsyncRuntime>().unwrap();
        let rt = rt.rt.clone();

        let ws = rt.block_on(async { connect_to_deepgram().await });

        let (tx, rx) = crossbeam_channel::unbounded();

        let (ws_tx, mut ws_rx) = ws.split();

        // Here we spawn an indefinite async task which receives websocket messages from Deepgram and pipes
        // them into a crossbeam channel, allowing the main synchronous Bevy runtime to access them when
        // needed (e.g. once per frame in the game loop).
        rt.spawn(async move {
            while let Some(Ok(message)) = ws_rx.next().await {
                let _ = tx.send(message);
            }
        });

        info!("Connected to Deepgram");

        DeepgramWebsocket { tx: ws_tx, rx }
    }
}

/// This uses the portaudio crate to get a connection with your computer's default audio input device (microphone).
/// It takes the sender half of a channel as an input because this function will spawn a thread which pipes audio
/// from the microphone to the receiving half of the channel. An example usage is:
/// ```
/// let (tx, rx) = crossbeam_channel::unbounded();
/// connect_to_microphone(tx);
/// while let Ok(audio) = rx.try_recv() {
///     // do something with the audio
/// }
/// ```
/// This is based on the following tutorial: https://dev.to/maniflames/audio-visualization-with-rust-4nhg
fn connect_to_microphone(tx: crossbeam_channel::Sender<Vec<f32>>) {
    let port_audio = portaudio::PortAudio::new().expect("Initializing PortAudio failed.");
    let mic_index = port_audio
        .default_input_device()
        .expect("Failed to get default input device.");
    let mic_info = port_audio
        .device_info(mic_index)
        .expect("Failed to get microphone info.");
    let input_params = portaudio::StreamParameters::<f32>::new(
        mic_index,
        1,
        true,
        mic_info.default_low_input_latency,
    );

    let input_settings =
        portaudio::InputStreamSettings::new(input_params, mic_info.default_sample_rate, 256);

    let (audio_sender, audio_receiver) = crossbeam_channel::unbounded();

    let audio_callback =
        move |portaudio::InputStreamCallbackArgs { buffer, .. }| match audio_sender.send(buffer) {
            Ok(_) => portaudio::Continue,
            Err(_) => portaudio::Complete,
        };

    let mut audio_stream = port_audio
        .open_non_blocking_stream(input_settings, audio_callback)
        .expect("Failed to create audio stream.");
    audio_stream.start().expect("Failed to start audio stream.");

    // Here we spawn an indefinite synchronous task in its own thread which receives audio from
    // the microphone and pipes it into a crossbeam channel allowing Bevy to access the audio
    // when needed (e.g. once per frame in the game loop) via the receiving half of the channel.
    std::thread::spawn(move || {
        while audio_stream.is_active().unwrap() {
            while let Ok(audio_buffer) = audio_receiver.try_recv() {
                let _ = tx.send(audio_buffer.to_owned());
            }
        }
    });
}

/// A helper function for converting f32 PCM samples to i16 (linear16) samples.
/// Deepgram currently does not support f32 PCM.
fn f32_to_i16(sample: f32) -> i16 {
    let sample = sample * 32768.0;

    // This is a saturating cast. For more details, see:
    // <https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast>.
    sample as i16
}

fn handle_asr(
    microphone_receiver: Res<MicrophoneReceiver>,
    mut deepgram_websocket: ResMut<DeepgramWebsocket>,
    async_runtime: Res<AsyncRuntime>,
    mut speech_events: EventWriter<SpeechEvent>,
) {
    while let Ok(audio_buffer) = microphone_receiver.rx.try_recv() {
        let sample_bytes = audio_buffer
            .into_iter()
            .flat_map(|sample| f32_to_i16(sample).to_le_bytes())
            .collect();

        let rt = async_runtime.rt.clone();

        let _ = rt.block_on(async {
            deepgram_websocket
                .tx
                .send(tungstenite::Message::Binary(sample_bytes))
                .await
        });
    }

    while let Ok(message) = deepgram_websocket.rx.try_recv() {
        if let tungstenite::Message::Text(message) = message {
            if message.contains("sugar") {
                info!("Sending sugar speech event");
                speech_events.send(SpeechEvent::Sugar);
            }
            if message.contains("mentos") {
                info!("Sending mentos speech event");
                speech_events.send(SpeechEvent::Mentos);
            }
            if message.contains("bridge") {
                info!("Sending bridge speech event");
                speech_events.send(SpeechEvent::Bridge);
            }
        }
    }
}
