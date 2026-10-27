# Fork 更新记录

这里仅记录本fork的适配与维护变化。原作者的版本历史见[上游README归档](archive/upstream-9767558/README.md)。

## 2026-10-27 — 文档与维护边界

- 重写fork首页，以Windows兼容性、分支选择和验证范围为主要入口。
- 原上游项目README、ALSA、Inferno2pipe和Docker集成测试文档完整归档；原入口保留导航。
- 增加兼容性说明，区分库能力、应用侧职责和未验证项目。
- 独立维护；GitHub镜像草稿PR #58已撤回。配置与验证经历采用概括、脱敏表述。

## 2026-10-27 — Windows 公开适配基线

源码提交`902745c`，位于`codex/windows-native-ptpv1`：

- 原生QPC/PTPv1 follower、时钟过期传播及TX重获映射后的时间线恢复。
- 网络与本地时间之间的仿射速率逆换算。
- 48 kHz、双通道TX/RX合成测试宿主。
- Windows/Linux自动化测试及Windows Release构建通过；MIT netdev0.32 vendor长度边界补丁保留许可。

这不是新发布版本或完整硬件认证；没有创建新的release/tag。
