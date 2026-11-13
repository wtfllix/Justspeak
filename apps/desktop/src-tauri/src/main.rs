#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::State;
use voice_core::{Device, Snapshot, VoiceCore};
#[tauri::command]
async fn microphones(core: State<'_, VoiceCore>) -> Result<Vec<Device>, String> {
    core.devices().await
}
#[tauri::command]
async fn connect(
    core: State<'_, VoiceCore>,
    endpoint: String,
    nickname: String,
    join_code: String,
    microphone: String,
) -> Result<(), String> {
    core.connect(endpoint, nickname, join_code, microphone)
        .await
}
#[tauri::command]
async fn disconnect(core: State<'_, VoiceCore>) {
    core.disconnect().await
}
#[tauri::command]
async fn set_muted(core: State<'_, VoiceCore>, muted: bool) -> Result<(), String> {
    core.set_muted(muted).await
}
#[tauri::command]
async fn select_microphone(core: State<'_, VoiceCore>, id: String) -> Result<(), String> {
    core.select_microphone(id).await
}
#[tauri::command]
async fn snapshot(core: State<'_, VoiceCore>) -> Snapshot {
    core.snapshot().await
}
fn main() {
    tauri::Builder::default()
        .manage(VoiceCore::default())
        .invoke_handler(tauri::generate_handler![
            microphones,
            connect,
            disconnect,
            set_muted,
            select_microphone,
            snapshot
        ])
        .run(tauri::generate_context!())
        .expect("桌面启动失败");
}
