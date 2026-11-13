# 音频路径与兼容性核实

2026-11-13 阅读完整 PRD v1.0（24 章）及嵌入草图。需求不可把 M0 替换为 WebView WebRTC；Windows 原生 Rust 媒体是基线。

## 依赖证据

- LiveKit Rust `=0.10.1`：crates.io 发布包的 `src/platform_audio/mod.rs`、`src/prelude.rs`、`examples/platform_audio.rs` 已核对。`PlatformAudio::recording_devices`、`set_recording_device`、`switch_recording_device`、`rtc_source`、`start_recording`、`stop_recording` 与 `Room::connect` / `publish_track` 是实现使用的 API。Windows 底层为 WASAPI。源于[官方 Rust SDK](https://github.com/livekit/rust-sdks)。SDK API 的存在不代表目标硬件已验证。
- `NativeAudioSource::capture_frame` 支持 48 kHz、单声道、10 ms（480 样本）PCM；零队列直接推送要求 10 ms。`NativeAudioStream` 可取解码 PCM。计划用独立无硬件探针验证，不能与真实采集验证混淆。
- LiveKit 信令默认未打开 WSS TLS feature，显式启用 `rustls-tls-native-roots`；Token 请求使用 reqwest Rustls。远程 API 限 HTTPS、媒体限 WSS；本机回环允许 HTTP/WS。禁止重定向，不绕过 TLS 验证。M2 Registry 的完整地址/身份校验未在 M0 实现。
- Tauri 2 与 React 只承担 UI/IPC。Tauri Linux 需要 GTK/WebKitGTK 开发依赖；Rust SDK 使用原生 libwebrtc，不依赖 WebView 的麦克风权限。
- `livekit-server-sdk 2.19.3` 的异步 `AccessToken.toJwt` 与 `TrackSource.MICROPHONE` 已经真实单元测试核对。

## 选择

原生路径：WASAPI → LiveKit PlatformAudio ADM → Device 音轨 → Opus / SFU → 自动订阅远端音轨 → ADM 播放。React 仅收状态与设备列表，音频不做 IPC 序列化。24 kbps 是配置目标，尚未实测编码负载与网卡带宽。

先采用同一 ADM 的采集/播放，为后续 AEC 播放参考留空间。当前不能宣称 AEC/AGC/标准降噪已经正确开启或有效：0.10.1 的 processing API 对桌面软件 APM 的效果仍需测试，不能只看布尔配置。AI/RNNoise、每成员音量均未接入。

自定义 PCM 作为独立探针路径验证，不与 PlatformAudio 并发混用。上游 basic_room 示例有混合 Device/NativeAudioSource 时序风险诊断，M0 不启用两条发布路径同时运行。生产采集/处理拓扑需在 Windows 实测后冻结。

## 生命周期

Rust 使用互斥会话与原子连接代次。新连接先关闭旧房间；取消会递增代次，在 Token 返回、Room 完成和发布完成后检查并清理。音轨初始静音，取消期间迟到发布不能开麦。Token 请求 10 秒、媒体连接 20 秒、发布 10 秒、关闭 5 秒超时。IPC 允许 UI 取消在途连接，UI 丢弃旧操作的返回值。

首次连接、麦克风切换、网络重连均保持静音，只有媒体状态 Connected 时用户可开麦。SDK 原生处理自动重连；重连彻底失败后须手动断开重新授权。UI 文案只报告媒体会话状态和真实成员/订阅数，不把它当作口到耳已通。

## 下一阶段接口

D01 将选中频道与当前成功频道分开；M1 加入频道事务与右键加入。
D02 私钥/挑战签名只放 Rust；M0 不持久身份、不按昵称赋权限。
D03 将 channelId 留在真实快照；M1 成功后保存并执行恢复/默认回退。
D04 `set_muted` 作为当前安全门；M1 将 PTT、VAD、撤权与设备门控合成。
D05 右侧面板控制媒体核心，不因 React 展开/重绘重连；高级诊断留折叠区域。

## 可复现的合成 PCM 探针

先独立启动测试 API 与 LiveKit，给进程设置 `POC_API_URL`（如本机 http://127.0.0.1:3001）与 `POC_JOIN_CODE`，运行 `node scripts/run-pcm-probe.mjs`。不向探针提供服务端签名 Secret。

探针连接两个真实 Rust Room，分别发布 48 kHz 单声道 10 ms 正弦波帧，从另一个 Room 的 NativeAudioStream 检查非零解码帧，最后断开。只验证 SDK PCM 接口、Opus/SFU/订阅链路，不验证任何真实声音、设备、AEC、双机通话或性能。默认不持久化音频。实际执行状态只以测试报告为准。
