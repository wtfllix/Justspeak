# xxspeak · M0 核心语音 PoC

开发基线是完整的 [PRD v1.0](xxspeak_PRD_v1.0.docx)。可搜索的[文本副本](docs/baseline/xxspeak-PRD-v1.0.txt)与[文档内原始草图](docs/baseline/original-sketch.png)已归档。范围仅 PRD §20 M0。**实现与验收分开：Windows 双机一小时通话、音质和性能尚未验证，M0 尚未通过退出门槛。**

客户端：Tauri 2 + React/TypeScript + Rust LiveKit 0.10.1。Rust 拥有 Token 请求、Room、PlatformAudio 原生设备、采集/播放与断开生命周期；PCM 不经过 React/WebView。服务端：Fastify 5 + LiveKit Token SDK，单节点 LiveKit。锁定 npm package-lock.json 与 Cargo.lock。

## 启动客户端（Windows 优先）

安装 Node.js 24 LTS、Rust stable MSVC、Visual Studio C++ Build Tools（C++ 桌面开发、Windows SDK）与 WebView2。参见 [Tauri 前置要求](https://v2.tauri.app/start/prerequisites/)。首次 Rust 构建会下载 SDK 使用的原生 libwebrtc，须允许访问 GitHub Releases；不要设置 DOCS_RS 绕过构建来冒充通过。

```powershell
npm ci
npm run desktop
```

正式编译（M0 只构建可执行程序，不制作安装器）：

```powershell
npm run tauri -w @xxspeak/desktop -- build --no-bundle
```

程序位于 `target/release/xxspeak-desktop.exe`。`npm run dev` 仅网页布局预览，无原生音频，界面会提示。Linux 需额外安装 Tauri 的 WebKitGTK 4.1/GTK3 开发库和 LiveKit 原生构建依赖；本轮未验证 Linux 桌面运行。

Windows x64 的 `.cargo/config.toml` 按 LiveKit 上游配置启用 `+crt-static`，让 Rust/C++ 桥接代码与预编译 libwebrtc 使用一致的 MSVC 运行库。遇到 `LNK2038 RuntimeLibrary MT_StaticRelease/MD_DynamicRelease` 时，先拉取包含此配置的最新代码，并检查是否设置了覆盖项目配置的 `RUSTFLAGS` 或 `CARGO_ENCODED_RUSTFLAGS`。不要通过忽略链接冲突来构建。

连接时展开设置、刷新并选择真实麦克风，填昵称、管理员给的 Token API 地址和内测连接码，点“仅本次连接”。两端都要点“开启麦克风”。连接码只在内存使用，成功后清空；切换麦克风或网络重连后保持静音，手动重新开麦。远端音频自动订阅并经原生播放设备播放；M0 选择列表中的第一个输出设备。断开后可再次连接，需重新输入连接码。连接取消最长等待在途网络超时，但迟到结果不得继续发布。

## 自托管服务（Linux Docker Engine + Compose v2）

从仓库根目录执行：

```sh
node scripts/init-deploy.mjs
# 编辑 deploy/.env：仅服务端读取，勿复制到客户端目录或 VITE_* 变量
# 必须先 npm ci，以确保本地开发依赖；镜像本身按 lockfile 安装服务端生产依赖
npm ci
docker compose --env-file deploy/.env -f deploy/compose.yaml up -d --build
docker compose --env-file deploy/.env -f deploy/compose.yaml ps
curl --fail http://127.0.0.1:3001/healthz
```

本机 API 是 `http://127.0.0.1:3001`，媒体是 `ws://127.0.0.1:7880`。host 网络配置面向 Linux VPS，未验证 Docker Desktop。API 健康检查只证明 Token API 响应，不证明音频可用。LiveKit 健康检查验证 HTTP 端点，不替代双机测试。

两台 Windows 连接远程 Linux 时，准备两个域名与受信任证书。将 `deploy/.env` 改为：

```dotenv
LIVEKIT_URL=wss://voice.your-domain.tld
USE_EXTERNAL_IP=true
API_DOMAIN=api.your-domain.tld
MEDIA_DOMAIN=voice.your-domain.tld
ACME_EMAIL=you@your-domain.tld
```

DNS 指向 VPS；开放 TCP 80/443（TLS 与 ACME）、TCP 7881（ICE TCP）、UDP 7882（UDP Mux）。NAT 场景还要配置相应映射，不能只开放 443。7880 与 3001 绑定回环，不公开管理接口。启动 TLS 代理：

```sh
docker compose --env-file deploy/.env -f deploy/compose.yaml --profile tls up -d --build
```

客户端填 `https://api.your-domain.tld`。Caddy 仅代理媒体 `/rtc` 与 `/rtc/*` 信令入口，其他路径拒绝；实际 SDK 新连/重连路径需实机核对。受限网络 TURN 未配置、未验证。参考 [LiveKit 部署](https://docs.livekit.io/transport/self-hosting/deployment/)与[端口说明](https://docs.livekit.io/transport/self-hosting/ports-firewall/)。

`LIVEKIT_API_SECRET` 是服务端签名密钥，只传入 API/LiveKit 容器。`POC_JOIN_CODE` 是受控内测准入码，由管理员单独分发给测试者，不是服务端签名密钥。Token 最长 5 分钟、固定房间、服务端随机身份，只授予麦克风发布与订阅，不允许视频、数据、管理权限。加入码每 IP 每分钟 20 次请求限流。**这不是 M1 持久身份或 M4 撤权实现**；已签 Token 的到期不终止已有连接，轮换连接码也不撤销已发 Token。只在受控内测环境使用。

Compose 固定 LiveKit 1.13.11、Node 24.16.0、Caddy 2.10.2；未承诺这些镜像经过本机 Docker 验证。服务端 M0 无 SQLite/用户数据库，重启清除运行会话；需要备份 `deploy/.env` 和 Caddy 证书卷。勿将它们提交版本库。停服务：

```sh
docker compose --env-file deploy/.env -f deploy/compose.yaml --profile tls down
```

不使用 Docker 的 Token API 开发：复制 `deploy/.env.example` 为 `apps/voice-server/.env`，填独立随机服务端密钥、连接码与媒体 URL，然后 `npm run server`。LiveKit 必须使用相同 API Key/Secret 独立启动；服务端 `.env` 不进入前端构建。

## 已有结构与后续边界

| 位置 | 职责 |
| --- | --- |
| apps/desktop/src | 草图连接表单、窄窗服务器/频道/真实成员、右侧设置 |
| apps/desktop/src-tauri | 受限本地 IPC、窗口、Rust 核心实例 |
| crates/voice-core | 原生媒体、麦克风、会话状态、取消/断开 |
| apps/voice-server | 内测 Token API、校验、音频权限、限流 |
| deploy | 单节点 Compose、可选 TLS 反代 |
| docs | 需求基线、兼容性选择、检查记录、实机验收计划 |

五项固定决策保持 PRD 原义：D01 频道单击只选中，双击加入入口留待 M1；D02 Rust 身份接口后续加入，M0 每连接随机受控身份；D03 成功频道 ID 已在核心快照中分离，持久化与回退留待 M1；D04 手动开麦/静音是安全发送门，PTT/VAD 未实现；D05 简洁麦克风面板与高级折叠已分区。书签、8 位服务器 ID、连接收藏均明确禁用并标阶段，没有虚假解析或演示在线成员。设置展开不重建 Rust Room。

## 检查与验收

```sh
npm run check
npm test
npm run build
npm audit
cargo fmt --all --check
cargo check --locked -p voice-core
cargo check --locked --workspace
```

实际执行结果见 [M0 测试报告](docs/M0-TEST-REPORT.md)，Windows 操作与性能采集见 [实机验收规程](docs/M0-TEST-PLAN.md)。不能以网页构建、API 测试或合成 PCM 证明两机通话验收通过。

## 排错

- 连接码错误：核对服务器 `POC_JOIN_CODE`，不要把 `LIVEKIT_API_SECRET` 填入客户端。
- API 无响应：检查 `/healthz`、域名/TLS、反向代理与防火墙；保留原输入重试。
- 信令已连接但双方无声：双方开麦、检查真实输入/输出设备与系统权限，检查 UDP 7882/TCP 7881、NAT 和远端订阅数。订阅数不代表可听音质。
- 麦克风切换失败：保持静音，刷新设备后再选。完整自动热插拔策略留待 M3。
- 重连终止：主动断开后重新输入连接码获取新 Token。Token 过期不应被解释为封禁生效。
- LiveKit UDP buffer 警告：按官方部署要求调整宿主机 socket buffer 后复测；本环境出现此警告，未调宿主配置。
