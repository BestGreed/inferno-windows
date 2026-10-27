# 兼容性与验证范围

## 分支边界

Windows 适配在 `codex/windows-native-ptpv1`；`dev` 目前仅包含上游基线及独立维护文档。请勿把 `dev` 的旧源码视为已有 Windows 后端。

| 项目 | 状态与证据 |
| --- | --- |
| Windows 原生库与双向合成宿主 | 公开适配基线902745c；MSVC托管及本机gnullvm构建、单元/模拟测试通过 |
| Windows PTPv1 | QPC单调时基；two-step Sync/Follow_Up/Delay_Req/Delay_Resp；指定master和网卡；软件时间戳 |
| Linux 音频库 | 保留usrvclock；Ubuntu库与时钟组件测试通过；未据此宣称本轮完成ALSA硬件验收 |
| macOS | 本fork未验证 |
| 发现、订阅、TX/RX、多播基础 | 继承自Inferno；本次未逐功能完成物理设备回归 |
| 系统音频端点 | 没有WASAPI/ASIO/VST3端点或Windows虚拟声卡实现 |
| PacketSource / 外部应用专用集成 | 不在当前公开适配中 |
| 完整Controller遥测、长期来源时钟漂移补偿 | 尚未完成 |

## 应用侧分工

应用可以负责系统信息查询、配置读写、前端接口、路由及调度；Inferno负责按明确参数执行发现、控制、时钟和媒体收发。示例只用于验证库的调用方式，不代表必须增加独立OS音频I/O层。

Windows时钟当前通过`CLOCK_PATH`接收`interface IPv4,master IPv4,local MAC`。API参数和错误报告仍可改善；没有宣称已形成完整设备管理API。PTPv2、BMCA、one-step Sync、硬件时间戳、自动换主尚未实现。

## 已完成的自动化验证

[对应902745c的运行记录](https://github.com/BestGreed/inferno/actions/runs/40500641157)：

- Windows/MSVC：89项音频库测试、14项时钟单元测试、1项临时回环UDP模拟主时钟测试；Release双向示例构建通过。
- Ubuntu：92项音频库测试、14项时钟单元测试、1项临时回环UDP模拟主时钟测试。
- 本机Windows gnullvm：相应库/时钟/UDP测试、Release示例、合成频差和QPC单调读取检查通过。

后续文档改动不扩大以上验证范围。合成数据拟合残差不是绝对同步精度；历史物理设备收发观察也不是当前公开提取版本的长时间验收。低延迟参数、源时钟与网络时钟失配、失锁/重获、实际设备互操作需分别测量。

完整构建与运行步骤见[Windows操作说明](https://github.com/BestGreed/inferno/blob/codex/windows-native-ptpv1/WINDOWS-TESTING.md)。
