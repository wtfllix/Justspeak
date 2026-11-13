# M0 内网服务器与 Windows SSH 隧道

适用于已有 Linux 服务器、两台测试电脑可直达服务器内网 IP、暂时没有 HTTPS/WSS 域名的受控 PoC。API/信令保持服务器回环监听；无需放宽客户端的 TLS 校验。

## 服务器

在 Linux 服务器项目目录执行。首次初始化凭据后不要重复生成或覆盖 `.env`。

```sh
node scripts/init-deploy.mjs
docker compose -p xxspeak-m0 --env-file deploy/.env -f deploy/compose.yaml config --quiet
docker compose -p xxspeak-m0 --env-file deploy/.env -f deploy/compose.yaml up -d --build
docker compose -p xxspeak-m0 --env-file deploy/.env -f deploy/compose.yaml ps
curl --fail http://127.0.0.1:3001/healthz
```

保持 `LIVEKIT_URL=ws://127.0.0.1:7880`、`USE_EXTERNAL_IP=false`。将 `LIVEKIT_NODE_IP` 设为两台测试电脑都可达的服务器内网 IP，避免多网卡主机误选 Docker 网桥地址。若有多个 Docker/VPN 网卡，还应将 `LIVEKIT_INTERFACE` 设为实际内网网卡名（通过 `ip -brief address` 查询）。服务器 TCP 3001/7880 绑定回环；测试电脑需能访问服务器的 ICE UDP 7882 或 TCP 7881。多网卡/VPN/NAT 的实际连通性必须测试，不能只看健康端点。

## 每台 Windows

先确保管理员提供的 SSH 账户可以登录。将 `SSH_USER@SERVER_IP` 替换为实际 SSH 登录目标。保留此 PowerShell 窗口：

```powershell
ssh -N -o ExitOnForwardFailure=yes -o ServerAliveInterval=30 -L 127.0.0.1:3001:127.0.0.1:3001 -L 127.0.0.1:7880:127.0.0.1:7880 SSH_USER@SERVER_IP
```

此命令正常运行后不输出内容；不要关闭。另开一个 PowerShell 验证：

```powershell
Invoke-RestMethod http://127.0.0.1:3001/healthz
```

管理员从服务器 `deploy/.env` 中只取 `POC_JOIN_CODE`，单独分发给测试者；不要把完整 `.env` 或 LiveKit API Secret 复制到客户端。管理员可运行：

```sh
sed -n 's/^POC_JOIN_CODE=//p' deploy/.env
```

客户端填 `http://127.0.0.1:3001`、昵称、连接码，选择真实麦克风并连接。两端都要手动开麦。若媒体已连接但无声，检查 ICE 端口、设备、双方静音状态和服务器候选地址。

## 边界

隧道只转发 API 与 WebSocket 信令，媒体走测试电脑到服务器的 WebRTC 链路，不通过 SSH 隧道转发。仅当两台测试电脑都能到达服务器的媒体地址时才可用。

这是一条受控内测路径，需要每台测试电脑拥有管理员提供的 SSH 访问方式。正式玩家连接应部署 README 中的 HTTPS/WSS 域名入口，不应向玩家分发 root 私钥。

## 更新与停止

```sh
git pull --ff-only
docker compose -p xxspeak-m0 --env-file deploy/.env -f deploy/compose.yaml up -d --build
# 停止 xxspeak，不触碰其他 Compose 项目
# docker compose -p xxspeak-m0 --env-file deploy/.env -f deploy/compose.yaml down
```

Windows 断开客户端后，在隧道窗口按 Ctrl+C 关闭 SSH 转发。双机持续通话、设备切换与性能仍按 M0-TEST-PLAN.md 实测。
