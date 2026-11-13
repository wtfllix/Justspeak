//! Synthetic PCM transport probe. NOT microphone/playback or two-machine acceptance.
use futures_util::StreamExt;
use livekit::{
    options::TrackPublishOptions,
    prelude::*,
    webrtc::{
        audio_frame::AudioFrame,
        audio_source::{native::NativeAudioSource, AudioSourceOptions},
        audio_stream::native::NativeAudioStream,
    },
};
use serde::Deserialize;
use std::{borrow::Cow, time::Duration};
use tokio::time::timeout;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Ticket {
    token: String,
    livekit_url: String,
}
async fn ticket(name: &str) -> Ticket {
    let endpoint = std::env::var("POC_API_URL").expect("POC_API_URL");
    let code = std::env::var("POC_JOIN_CODE").expect("POC_JOIN_CODE (not LiveKit secret)");
    reqwest::Client::new()
        .post(format!("{endpoint}/v1/poc/token"))
        .json(&serde_json::json!({"nickname":name,"joinCode":code}))
        .send()
        .await
        .expect("API")
        .error_for_status()
        .expect("authorization")
        .json()
        .await
        .expect("ticket")
}
async fn subscribe(rx: &mut tokio::sync::mpsc::UnboundedReceiver<RoomEvent>) -> NativeAudioStream {
    timeout(Duration::from_secs(20), async {
        while let Some(event) = rx.recv().await {
            if let RoomEvent::TrackSubscribed {
                track: RemoteTrack::Audio(track),
                ..
            } = event
            {
                return NativeAudioStream::new(track.rtc_track(), 48000, 1);
            }
        }
        panic!("events ended")
    })
    .await
    .expect("subscription timeout")
}
async fn receive(mut stream: NativeAudioStream) -> usize {
    timeout(Duration::from_secs(10), async {
        let mut count = 0;
        while let Some(frame) = stream.next().await {
            if frame.data.iter().any(|s| s.abs() > 50) {
                count += 1;
            }
            if count >= 50 {
                return count;
            }
        }
        panic!("stream ended")
    })
    .await
    .expect("no nonzero decoded PCM")
}
#[tokio::main]
async fn main() {
    let a = ticket("PCM probe A").await;
    let b = ticket("PCM probe B").await;
    let (ra, mut ea) = Room::connect(&a.livekit_url, &a.token, RoomOptions::default())
        .await
        .expect("room A");
    let (rb, mut eb) = Room::connect(&b.livekit_url, &b.token, RoomOptions::default())
        .await
        .expect("room B");
    let sa = NativeAudioSource::new(AudioSourceOptions::default(), 48000, 1, 0);
    let sb = NativeAudioSource::new(AudioSourceOptions::default(), 48000, 1, 0);
    for (room, source) in [(&ra, &sa), (&rb, &sb)] {
        room.local_participant()
            .publish_track(
                LocalTrack::Audio(LocalAudioTrack::create_audio_track(
                    "synthetic-pcm",
                    RtcAudioSource::Native(source.clone()),
                )),
                TrackPublishOptions {
                    source: TrackSource::Microphone,
                    ..Default::default()
                },
            )
            .await
            .expect("publish");
    }
    let (xa, xb) = tokio::join!(subscribe(&mut ea), subscribe(&mut eb));
    let task_a = tokio::spawn(receive(xa));
    let task_b = tokio::spawn(receive(xb));
    let mut ticker = tokio::time::interval(Duration::from_millis(10));
    for i in 0..300 {
        ticker.tick().await;
        let samples: Vec<i16> = (0..480)
            .map(|n| {
                (((i * 480 + n) as f32 * 440.0 * std::f32::consts::TAU / 48000.0).sin() * 8000.0)
                    as i16
            })
            .collect();
        let frame = AudioFrame {
            data: Cow::Owned(samples),
            sample_rate: 48000,
            num_channels: 1,
            samples_per_channel: 480,
        };
        sa.capture_frame(&frame).await.expect("push A");
        sb.capture_frame(&frame).await.expect("push B");
    }
    let ca = task_a.await.expect("A receiver");
    let cb = task_b.await.expect("B receiver");
    ra.close().await.expect("close A");
    rb.close().await.expect("close B");
    println!("Synthetic PCM transport passed: A received {ca} nonzero frames, B received {cb}. NOT real voice acceptance.");
}
