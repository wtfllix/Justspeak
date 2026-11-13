# M0 测试报告 · 2026-11-13

**结论：M0 实现已落地，未通过 M0 实机退出门槛。** Windows 双机连续一小时通话及性能均未执行，不能以本文其他检查替代。

## 环境与边界

仓库初始只有 xxspeak_PRD_v1.0.docx，无已有代码或 Git 元数据。已完整读取 24 章并提取原始草图；归档于 docs/baseline，含源文件和草图 SHA256。

实际开发主机 Ubuntu 26.04 x86_64，Node 24.16.0 / npm 11.13.0，初始无 Rust、Docker、Tauri GTK/WebKit 开发库。已安装 Rust 1.100.0；为原生 SDK Linux 编译，在 /tmp 独立解包 Clang 21.1.8，未使用 sudo 修改系统包。完整桌面编译仍受系统库缺失阻塞。原生设备探针真实枚举到两个 RDP Source 虚拟输入（见 evidence/native-devices.txt），无已确认的实体录音设备；没有可访问的两台 Windows 测试电脑。

## 已执行

| 检查 | 实际结果 | 证据/口径 |
| --- | --- | --- |
| `npm run check` | 通过 | React 严格 TypeScript、服务端 JS 语法 |
| `npm test` | 4 组全部通过 | 配置失败关闭、JWT 真签名验证、音频权限/固定房间/随机身份/TTL、输入/连接码拒绝、限流 |
| `npm run build` | 通过 | Vite 7.3.8 前端生产构建；不是 Tauri Windows exe |
| `npm audit` | 0 项漏洞 | evidence/npm-audit.txt；只涵盖 npm 已知公告 |
| 客户端边界静态扫描 | 通过 | scripts/check-client-boundary.mjs 扫前端源码、产物与 Rust 客户端，不含服务端密钥配置引用；尚无 Windows 二进制扫描 |
| `cargo check --locked -p voice-core --all-targets` | 通过 | Rust SDK 原生库真实编译，未用 DOCS_RS 跳过 |
| `cargo test --locked -p voice-core` | 1 项通过 | endpoint TLS/回环/凭据/查询串边界；evidence/native-tests.txt |
| `cargo clippy --locked -p voice-core --all-targets -- -D warnings` | 通过 | evidence/native-clippy.txt |
| 原生设备枚举探针 | 通过，仅虚拟设备 | 两个 RDP Source 输入，不作 Windows 设备验收 |
| `cargo fmt --all --check` | 通过 | 实际通过的命令使用 +stable，版本同锁定 1.100.0 |
| `cargo check --locked --workspace` | 未通过：环境缺依赖 | 缺少 gdk-3.0、GTK/WebKitGTK 开发包，未验证 Tauri 壳编译 |
| LiveKit 1.13.11 真服务启动 | 通过 | 原版发布二进制读取与 Compose 相同结构的配置；本机 17880/17881/17882 避免端口冲突 |
| API/LiveKit HTTP 集成 | 通过 | API 签发 Token，RoomServiceClient 真实服务鉴权建房、查询、删房 |
| Rust 双向合成 PCM 探针 | 通过，仅接口/传输验证 | 两个真实 Room、48 kHz 单声道 10ms 帧，各收 50 帧非零解码 PCM；evidence/pcm-probe.txt |
| 浏览器界面预览 | 通过，仅布局 | Chromium 340px 表单、680px 设置开合、无横向溢出/页面异常；网页音频入口禁用；evidence/*preview.png |
| 镜像标签核实 | 通过 | Docker Hub 三个 manifest HTTP 200，配置已锁 digest；未拉取/运行 |
| Compose YAML 静态解析 | 通过 | PyYAML 解析服务结构；不是 Docker Compose config 或容器运行 |

初轮发现 Vite 旧版本安全公告，已升级 7.3.8 并重跑 audit。Token 测试使用 SDK 实际提供的 exp 检查 TTL。PCM 探针按发布版封装的三参数 NativeAudioStream::new 修正后真实运行通过。检查失败不计作通过，修正后的结果单独列出。

本机 SDK 编译使用 `CXX=/tmp/xxspeak-clang/root/usr/bin/clang++-21` 与该目录动态库路径，Cargo 使用 `+stable` 选中本机已安装的同版本工具链。正式项目 rust-toolchain.toml 固定 1.100.0；package-lock.json 与 Cargo.lock 已生成。

## 未验证与未交付门槛

| 项目 | 状态与原因 | 后续执行 |
| --- | --- | --- |
| 两台 Windows 一小时真实语音 | 未验证，无测试机 | 按 M0-TEST-PLAN.md 双方真实持续通话并记时间线 |
| WASAPI 真实麦克风/扬声器、设备切换 | 未验证，无目标硬件 | USB/有线/蓝牙分别测试，默认输出路由确认 |
| Windows exe 编译、安装运行 | 未验证，无 Windows 构建环境 | 已写 Windows CI，但没有远程运行或 CI 成功记录 |
| 网络断开恢复、取消竞态 | 逻辑已实现，实机未验证 | 断网 5/15/60 秒、重复取消连接、确认无迟到采集 |
| CPU/私有工作集/端到端延迟/带宽 | 未测，无真实通话样本 | Windows 脚本输出原始 CSV；延迟另做口到耳测量 |
| 测量脚本 | 已提供，未在 Windows 验证 | 核对 CIM 计数器与子进程树后运行 |
| Docker Compose 镜像构建、健康检查 | 未验证，无 Docker | Linux VPS 实际 up --build、ps、健康检查与跨机媒体 |
| 公网 TLS、NAT、ICE TCP、TURN | 未验证 | 已有 TLS profile 与端口文档；TURN 未配置 |
| AEC/AGC/降噪效果 | 未验证 | 不因 SDK API 存在宣称降噪/回声消除通过；AI/RNNoise 属 M3 |
| 性能目标优于其他产品 | 无依据，不作声明 | 同环境实测对照，不以 RTT 或配置码率替代 |

真实 LiveKit 本机测试出现 UDP 接收缓冲不足生产建议值的警告；未修改宿主机参数。断开时出现数据通道 Close/abort 日志，PCM 探针正常退出。没有将这些日志解释为音质验收成功。

## 后续任务

先完成 Windows 构建、真实音频设备与双机一小时测试，补齐 M0 原生路径/AEC/PCM 决策及性能基线，再决定是否退出 M0。生产音频路径暂选 PlatformAudio；PCM 接入可行性已有 Linux 传输证据，Windows 设备与处理参考仍需确认。

M1：多频道事务、D01 双击加入、本地身份/挑战签名、D03 上次频道回退、D04 全局 PTT/VAD、个人音量与托盘。M2：Registry/服务器 ID/书签/密码。M3：完整设备热插拔、输出选择、试听、RNNoise 与 AEC 验证。M4/M5 安全管理、撤权与正式发布不提前纳入本轮。


## 后续验证：Windows CI 与 Linux Docker 部署

Windows 构建在 [运行 43439185786](https://github.com/wtfllix/Justspeak/actions/runs/43439185786) 已全部通过（提交 2e3056d）：JavaScript 检查、Windows 完整 Cargo workspace check 和 Tauri Release build --no-bundle。原先两个带 State 引用的异步命令改为返回 Result；Windows x64 CRT 按 LiveKit 上游配置统一为静态。证据见 evidence/windows-ci.json。此结果更新上文“Windows exe 编译未验证”的历史状态，仍不证明实际 Windows 音频设备或双机通话通过。

经用户授权，在另一台 Debian 13 服务器通过 Docker 29.6.1 / Compose 5.2.0 部署 Fastify 与 LiveKit 1.13.11，项目目录 /opt/xxspeak，Compose 项目名 xxspeak-m0。镜像构建成功，两个容器均 healthy；API/信令仅监听回环。凭据在服务器 deploy/.env，权限 600，未复制到客户端或版本库。证据见 evidence/remote-compose-health.txt。

多网卡主机初次自动选择了 Docker 网桥媒体地址。仅指定 NODE_IP 后的首轮跨机 PCM 仍因 ICE 超时失败；主机间 TCP/UDP 基础可达性检查成功。进一步通过 LIVEKIT_INTERFACE 将 ICE 绑定到实际内网网卡后，远端双向合成 PCM 探针通过，两端各收到 50 帧非零解码音频（evidence/remote-docker-pcm.txt）。新增可选 LIVEKIT_NODE_IP 与 LIVEKIT_INTERFACE 配置支持此部署。失败记录未计为通过。

这项测试是两个 Rust Room 到远端 Docker SFU 的合成音频链路，API/信令经 SSH 隧道，媒体经真实内网 UDP；并非两台 Windows 电脑的真人麦克风/扬声器通话。Windows 用户截图只确认窗口可启动与 HyperX 设备可枚举。两台 Windows 连续一小时、设备切换/重连、CPU/私有工作集和端到端延迟仍未测试，M0 退出门槛仍未满足。

尚未配置公网域名/TLS/TURN。当前受控内网测试按 DEPLOY-SSH-POC.md 使用 SSH 转发；每台测试电脑必须具备管理员提供的 SSH 登录方式，并能到达服务器的 ICE 地址。服务端出现 UDP buffer 生产建议值警告，未更改共享宿主机内核参数。
