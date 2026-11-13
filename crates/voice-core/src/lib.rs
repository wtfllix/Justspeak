use livekit::{options::TrackPublishOptions, prelude::*};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{sync::Mutex, task::JoinHandle, time::timeout};

#[derive(Serialize)]
pub struct Device {
    pub id: String,
    pub name: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: String,
    pub name: String,
    pub speaking: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub phase: String,
    pub channel_id: Option<String>,
    pub members: Vec<Member>,
    pub muted: bool,
    pub subscribed_audio: usize,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Ticket {
    token: String,
    livekit_url: String,
    channel_id: String,
}
struct Session {
    room: Arc<Room>,
    audio: PlatformAudio,
    track: LocalAudioTrack,
    events: JoinHandle<()>,
    channel: String,
    subscribed: Arc<AtomicUsize>,
}
#[derive(Default)]
pub struct VoiceCore {
    session: Mutex<Option<Session>>,
    generation: AtomicU64,
}
fn failure(message: &str) -> String {
    message.to_owned()
}
fn validate_endpoint(endpoint: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(endpoint).map_err(|_| failure("API 地址格式无效"))?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !(url.scheme() == "https" || url.scheme() == "http" && local)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(failure("API 必须使用 HTTPS；仅本机测试允许 HTTP"));
    }
    Ok(url)
}
impl VoiceCore {
    pub async fn devices(&self) -> Result<Vec<Device>, String> {
        let guard = self.session.lock().await;
        let audio = match guard.as_ref() {
            Some(s) => s.audio.clone(),
            None => PlatformAudio::new()
                .map_err(|_| failure("无法初始化音频设备，请检查设备与麦克风权限"))?,
        };
        let devices = audio
            .recording_devices()
            .map(|d| Device {
                id: d.id.to_string(),
                name: d.name,
            })
            .collect();
        if guard.is_none() {
            let _ = audio.stop_recording();
        }
        Ok(devices)
    }
    pub async fn connect(
        &self,
        endpoint: String,
        nickname: String,
        join_code: String,
        microphone: String,
    ) -> Result<(), String> {
        let endpoint = validate_endpoint(&endpoint)?;
        if nickname.trim().is_empty() || nickname.chars().count() > 32 || join_code.len() > 256 {
            return Err(failure("请检查昵称和内测连接码"));
        }
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let mut guard = self.session.lock().await;
        if let Some(old) = guard.take() {
            Self::close(old).await;
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| failure("无法初始化网络"))?;
        let response = client
            .post(
                endpoint
                    .join("/v1/poc/token")
                    .map_err(|_| failure("API 地址无效"))?,
            )
            .json(&serde_json::json!({"nickname":nickname.trim(),"joinCode":join_code}))
            .send()
            .await
            .map_err(|_| failure("Token API 无响应，请检查地址、TLS 和网络"))?;
        if !response.status().is_success() {
            return Err(failure(if response.status().as_u16() == 401 {
                "内测连接码不正确"
            } else {
                "Token 请求被拒绝，请检查服务配置或稍后重试"
            }));
        }
        let ticket: Ticket = response
            .json()
            .await
            .map_err(|_| failure("Token API 响应格式无效"))?;
        let media = url::Url::parse(&ticket.livekit_url).map_err(|_| failure("媒体地址无效"))?;
        if !(media.scheme() == "wss"
            || media.scheme() == "ws"
                && matches!(media.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")))
            || !media.username().is_empty()
            || media.password().is_some()
        {
            return Err(failure("远程媒体必须使用 WSS"));
        }
        if generation != self.generation.load(Ordering::SeqCst) {
            return Err(failure("连接已取消"));
        }
        let audio = PlatformAudio::new().map_err(|_| failure("无法初始化音频设备"))?;
        let device = audio
            .recording_devices()
            .find(|d| d.id.as_str() == microphone)
            .ok_or_else(|| failure("所选麦克风不可用，请刷新设备"))?;
        audio
            .set_recording_device(&device.id)
            .map_err(|_| failure("麦克风选择失败"))?;
        let speaker = audio
            .playout_devices()
            .next()
            .ok_or_else(|| failure("未找到播放设备"))?;
        audio
            .set_playout_device(&speaker.id)
            .map_err(|_| failure("播放设备初始化失败"))?;
        audio
            .stop_recording()
            .map_err(|_| failure("无法停止采集"))?;
        let (room, mut rx) = timeout(
            Duration::from_secs(20),
            Room::connect(&ticket.livekit_url, &ticket.token, RoomOptions::default()),
        )
        .await
        .map_err(|_| failure("媒体连接超时"))?
        .map_err(|_| failure("媒体连接失败，请检查 ICE 端口与服务配置"))?;
        let room = Arc::new(room);
        if generation != self.generation.load(Ordering::SeqCst) {
            let _ = room.close().await;
            return Err(failure("连接已取消"));
        }
        let track = LocalAudioTrack::create_audio_track("microphone", audio.rtc_source());
        track.mute();
        let publish = timeout(
            Duration::from_secs(10),
            room.local_participant().publish_track(
                LocalTrack::Audio(track.clone()),
                TrackPublishOptions {
                    source: TrackSource::Microphone,
                    audio_encoding: Some(livekit::options::AudioEncoding {
                        max_bitrate: 24_000,
                    }),
                    ..Default::default()
                },
            ),
        )
        .await;
        if !matches!(publish, Ok(Ok(_))) || generation != self.generation.load(Ordering::SeqCst) {
            let _ = audio.stop_recording();
            let _ = room.close().await;
            return Err(failure("发布失败或连接已取消"));
        }
        let subscribed = Arc::new(AtomicUsize::new(0));
        let count = subscribed.clone();
        let event_track = track.clone();
        let event_audio = audio.clone();
        let events = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    RoomEvent::TrackSubscribed {
                        track: RemoteTrack::Audio(_),
                        ..
                    } => {
                        count.fetch_add(1, Ordering::Relaxed);
                    }
                    RoomEvent::TrackUnsubscribed {
                        track: RemoteTrack::Audio(_),
                        ..
                    } => {
                        let _ = count.try_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                            Some(n.saturating_sub(1))
                        });
                    }
                    RoomEvent::Reconnecting => {
                        event_track.mute();
                        let _ = event_audio.stop_recording();
                    }
                    RoomEvent::Disconnected { .. } => {
                        event_track.mute();
                        let _ = event_audio.stop_recording();
                        break;
                    }
                    _ => {}
                }
            }
        });
        *guard = Some(Session {
            room,
            audio,
            track,
            events,
            channel: ticket.channel_id,
            subscribed,
        });
        Ok(())
    }
    async fn close(s: Session) {
        s.track.mute();
        let _ = s.audio.stop_recording();
        let _ = timeout(Duration::from_secs(5), s.room.close()).await;
        s.events.abort();
    }
    pub async fn disconnect(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        if let Some(s) = self.session.lock().await.take() {
            Self::close(s).await;
        }
    }
    pub async fn set_muted(&self, muted: bool) -> Result<(), String> {
        let guard = self.session.lock().await;
        let s = guard.as_ref().ok_or_else(|| failure("尚未连接"))?;
        if muted {
            s.track.mute();
            s.audio
                .stop_recording()
                .map_err(|_| failure("采集停止失败，请断开连接"))?;
        } else {
            if s.room.connection_state() != ConnectionState::Connected {
                return Err(failure("媒体未连接，不能开麦"));
            }
            s.audio
                .start_recording()
                .map_err(|_| failure("麦克风启动失败，请检查权限"))?;
            s.track.unmute();
        }
        Ok(())
    }
    pub async fn select_microphone(&self, id: String) -> Result<(), String> {
        let guard = self.session.lock().await;
        let s = guard.as_ref().ok_or_else(|| failure("尚未连接"))?;
        s.track.mute();
        let _ = s.audio.stop_recording();
        let device = s
            .audio
            .recording_devices()
            .find(|d| d.id.as_str() == id)
            .ok_or_else(|| failure("麦克风已不可用"))?;
        s.audio
            .switch_recording_device(&device.id)
            .map_err(|_| failure("切换麦克风失败，已保持静音"))?;
        let _ = s.audio.stop_recording();
        Ok(())
    }
    pub async fn snapshot(&self) -> Snapshot {
        let guard = self.session.lock().await;
        match guard.as_ref() {
            None => Snapshot {
                phase: "disconnected".into(),
                channel_id: None,
                members: vec![],
                muted: true,
                subscribed_audio: 0,
            },
            Some(s) => {
                let mut members: Vec<Member> = s
                    .room
                    .remote_participants()
                    .values()
                    .map(|p| Member {
                        id: p.identity().to_string(),
                        name: p.name(),
                        speaking: p.is_speaking(),
                    })
                    .collect();
                let p = s.room.local_participant();
                members.push(Member {
                    id: p.identity().to_string(),
                    name: format!("{}（我）", p.name()),
                    speaking: p.is_speaking() && !s.track.is_muted(),
                });
                Snapshot {
                    phase: match s.room.connection_state() {
                        ConnectionState::Connected => "connected",
                        ConnectionState::Reconnecting => "reconnecting",
                        _ => "disconnected",
                    }
                    .into(),
                    channel_id: Some(s.channel.clone()),
                    members,
                    muted: s.track.is_muted(),
                    subscribed_audio: s.subscribed.load(Ordering::Relaxed),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_endpoint;
    #[test]
    fn direct_endpoint_transport_boundary() {
        for value in [
            "http://127.0.0.1:3001",
            "http://localhost:3001",
            "https://api.example.com",
        ] {
            assert!(validate_endpoint(value).is_ok());
        }
        for value in [
            "http://public.example.com",
            "file:///etc/passwd",
            "https://user:pass@example.com",
            "https://example.com?token=secret",
            "https://example.com#fragment",
        ] {
            assert!(validate_endpoint(value).is_err());
        }
    }
}
