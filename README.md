# Inferno Windows compatibility fork

这是基于 [Inferno](https://gitlab.com/lumifaza/inferno) 独立维护的兼容性适配 fork，重点是将现有发现、控制与双向音频协议用于 Windows 原生宿主。原项目由 Teodor Wozniak（Teo）及贡献者开发；本 fork 保留原有署名和许可证。

This independently maintained fork adds an experimental native Windows QPC/PTPv1 follower and a duplex test host to Inferno. The protocol and media implementation originates from upstream Inferno. This is unofficial Dante-compatible software and is not endorsed by Audinate.

## 当前分支与入口

- **Windows 适配源码**：[`codex/windows-native-ptpv1`](https://github.com/BestGreed/inferno/tree/codex/windows-native-ptpv1)。构建前请切换到该分支。
- **`dev` 分支**：保留上游基线与本 fork 文档；目前不包含 Windows 适配代码。
- [兼容性与验证范围](docs/compatibility.md)
- [Windows 构建与测试操作](https://github.com/BestGreed/inferno/blob/codex/windows-native-ptpv1/WINDOWS-TESTING.md)
- [时钟实现与实验限制](https://github.com/BestGreed/inferno/blob/codex/windows-native-ptpv1/windows-clock/README.md)
- [更新记录](docs/CHANGELOG.md) · [维护说明](docs/maintenance.md) · [原上游文档归档](docs/archive/README.md)

## 适配范围

本 fork 的公开 Windows 分支包含：

- 进程内 QPC/PTPv1 从时钟，显式选择本机 IPv4、主时钟 IPv4 与本机 MAC。
- 四时间戳交换、时钟映射有效期检查，以及时钟丢失后恢复 TX 时间线。
- 复用 Inferno 原有发现、订阅、控制和 UDP TX/RX；示例提供 48 kHz、2 TX / 2 RX 合成音测试宿主。
- Windows/MSVC 与 Linux 的构建和测试检查；Linux 音频库仍使用原有 usrvclock 路径。

此层负责协议与媒体执行，可由外部应用提供 PCM，并管理配置与调度。它不是 Windows 系统虚拟声卡，也没有实现 WASAPI、ASIO 或 VST3 音频端点。公开适配不依赖 DVS/ASIO API，不修改系统时钟或执行主时钟选举。

## 获取与构建

```powershell
git clone --recurse-submodules --branch codex/windows-native-ptpv1 https://github.com/BestGreed/inferno.git
cd inferno
cargo build --locked --release -p inferno_aoip --example windows_loopback
```

需要标准 Windows Rust 工具链及对应链接器。不要在 Windows 上直接构建整个包含 ALSA 的 workspace。网络测试参数、端口和操作步骤见上述 Windows 文档。

## 验证状态与限制

公开代码基线 `902745c` 已通过 [Windows/MSVC 与 Ubuntu 托管检查](https://github.com/BestGreed/inferno/actions/runs/40500641157)：Windows 89 项库测试、Linux 92 项库测试，另有各平台 14 项时钟单元测试与 1 项 UDP 模拟主时钟测试；Windows Release 示例构建通过。

这些结果证明的是对应提交的构建和自动化测试，不等同于持续硬件互操作验收。历史开发中的物理设备观察与该公开基线的验证分开记录。绝对同步精度、长期稳定性、完整 Controller 遥测和独立来源时钟的漂移补偿仍需进一步工作；不声称稳定 250 µs TX。macOS 尚未在本 fork 验证。

## 来源与许可

上游主仓库为 [GitLab / lumifaza/inferno](https://gitlab.com/lumifaza/inferno)，[teodly/inferno](https://github.com/teodly/inferno) 是其 GitHub 镜像。本 fork 的原上游文档已按基线归档，不作为 Windows 适配的当前操作指南。

项目保留 **GPL-3.0-or-later OR AGPL-3.0-or-later** 双许可选择，见 [LICENSE](LICENSE)、[GPL 原文](inferno_aoip/LICENSE-GPL) 与 [AGPL 原文](inferno_aoip/LICENSE-AGPL)。Windows 分支中 vendored netdev 保留其 MIT 许可证及补丁说明。Dante 是 Audinate 的商标。
