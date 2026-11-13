# M0 交付说明

代码实现完成，M0 尚未验收通过。请先阅读 [README](../README.md) 的环境、启动和部署步骤。

## 启动

Windows 安装 Node 24、Rust MSVC、C++ Build Tools 与 WebView2 后，在仓库根目录执行 `npm ci`、`npm run desktop`。网页 `npm run dev` 只用于布局预览。

Linux 服务器首次运行 `node scripts/init-deploy.mjs`，编辑 `deploy/.env`，随后运行：

```sh
docker compose --env-file deploy/.env -f deploy/compose.yaml up -d --build
```

双机远程测试须配置 HTTPS/WSS 域名和防火墙，再加 `--profile tls` 启动代理。客户端只填写 Token API 地址、昵称、内测连接码与麦克风。LiveKit 签名 Secret 仅存于服务器，不能填入客户端。连接成功后两端手动开麦。

## 已执行的检查

前端类型检查/生产构建、4 组 Token API 测试、npm audit（0 漏洞）、Rust core check/test/clippy/fmt 已通过。原生枚举得到 RDP 虚拟设备。真实 LiveKit API 集成和双向合成 PCM 传输通过；仅证明接口与传输，不是设备/双机验收。浏览器窄窗与设置开合布局检查通过。

## 未验证项目

完整 Tauri 编译受 Linux GTK/WebKit 开发库缺失阻塞；Windows exe、真实 WASAPI 采集播放、双机一小时通话、麦克风切换/重连、Docker 容器部署、公网 TLS/ICE/TURN、CPU/私有工作集/口到耳延迟均未验证。Windows CI 和性能脚本已提供，但未运行 Windows 验证。

详见 [真实测试报告](M0-TEST-REPORT.md)、[实机规程](M0-TEST-PLAN.md) 和 [音频路径选择](M0-ARCHITECTURE.md)。未测项目没有填任何模拟性能数值。

## 下一步

先在两台 Windows 完成构建与持续一小时真实通话、设备切换/断开重连和性能采集，保存原始证据后判断 M0 是否退出。随后 M1 接入身份、频道事务、上次频道恢复、PTT 和托盘；M2–M5 按 PRD 推进，不在本轮提前实现。
