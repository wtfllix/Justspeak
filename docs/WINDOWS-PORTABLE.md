# xxspeak M0 · Windows x64 测试版

解压下载的产物，运行 `xxspeak.exe`。这是 M0 免安装测试程序；目标 Windows 必须有 Microsoft Edge WebView2 Runtime。运行程序不需要 Node.js、Rust 或源码。

产物包含 EXE、SHA256SUMS.txt、SOURCE_COMMIT.txt 和本说明。前端已嵌入 Release EXE，不需要启动 Vite。校验文件记录 EXE 的 SHA256，源码提交记录在 SOURCE_COMMIT.txt。

```powershell
Get-FileHash .\xxspeak.exe -Algorithm SHA256
```

## 连接当前内网测试服务器

使用管理员提供的 SSH 登录方式，在 PowerShell 建立到 Linux 服务器的隧道。将 SSH_USER@SERVER_IP 替换为实际登录目标，保持这个窗口打开：

```powershell
ssh -N -o ExitOnForwardFailure=yes -o ServerAliveInterval=30 -L 127.0.0.1:3001:127.0.0.1:3001 -L 127.0.0.1:7880:127.0.0.1:7880 SSH_USER@SERVER_IP
```

在客户端填写 `http://127.0.0.1:3001`、昵称和管理员提供的内测连接码，选择麦克风，点击“仅本次连接”，再手动开麦。LiveKit 签名 Secret 不属于客户端连接码，程序和本产物不包含部署凭据。

## 虚拟机测试

宿主机和 Windows 虚拟机各运行一份程序，使用不同昵称。每个 Windows 系统分别建立 SSH 隧道；虚拟机中的 127.0.0.1 指虚拟机自身。两端必须能访问服务器的实际媒体地址。

先确认虚拟机有可用的麦克风与播放设备。佩戴耳机；只有一套输入设备时，可轮流测试两个方向。音频重定向、USB/蓝牙与网络受虚拟化环境影响，记录时标注具体环境。

## 验证范围

流水线执行 Windows 编译、Release EXE 生成及 5 秒进程启动存活检查。进程存活不等于 UI、设备、通话或性能已通过验收。程序未提供安装器、自动更新与发布签名；这些属于后续里程碑。

M0 双机连续一小时真人通话、设备切换/重连与性能必须另行实测。虚拟机结果标注宿主机与虚拟机配置，不将估算写成验收数据。
