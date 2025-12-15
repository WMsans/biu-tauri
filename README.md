<h1 align="center">【Fork】Bilibili音乐播放器 (Biu)</h1>

> ⚠️ **关于此 Fork (About this Fork)**
>
> 本项目是 [wood3n/biu](https://github.com/wood3n/biu) 的 Fork 版本。
>
> **Fork 目的 / Purpose:**
> 此分支的主要目的是个人学习与研究Tauri V2后端，在尽可能少的修改前端代码的同时减少内存和CPU使用。
>
> **Fork Modifications:**
> * 使用Tauri V2 和 Rust 重写后端

<p align="center">
  <img src="./screenshots/logo.svg" alt="Biu logo" width="120" />
</p>
<p align="center">
  基于 Bilibili API 的跨平台桌面音乐播放器 🎧🎶
</p>
<p align="center">
  <a href="https://github.com/wood3n/biu/releases">
    <img src="https://img.shields.io/github/v/release/wood3n/biu?include_prereleases&label=最新版本&color=blueviolet" alt="Latest Version" />
  </a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-PolyForm%20Noncommercial%201.0.0-orange.svg" alt="License" /></a>
</p>

<table>
  <tr>
    <td width="50%" align="center">
      <img src="./screenshots/home.png" alt="home" width="100%" />
    </td>
    <td width="50%" align="center">
      <img src="./screenshots/main.png" alt="main" width="100%" />
    </td>
  </tr>
</table>

---

## 🛠️ 本地开发与构建 (Build & Run)

如果你想在本地运行或编译此 Fork 版本，请按照以下步骤操作：

### 1. 环境准备 (Prerequisites)
请确保你的系统已安装以下环境：
- **Node.js**: 建议使用最新的 LTS 版本。
- **pnpm**: 本项目使用 pnpm 作为包管理器 (`npm install -g pnpm`)。
- **Rust & Cargo**: Tauri 开发必须环境 (请参考 [Tauri 前置要求](https://tauri.app/v1/guides/getting-started/prerequisites))。

### 2. 安装依赖 (Install Dependencies)
在项目根目录下运行：
```bash
pnpm install
```
### 3. 开发模式运行 (Development)
启动本地开发环境（包含前端热更新与 Tauri 窗口）：
```bash
# 启动 Tauri 桌面应用开发模式
pnpm tauri dev
```
>注意：pnpm dev (即 rsbuild dev) 仅启动前端服务，如果需要调试桌面端 API，请使用 pnpm tauri dev。
### 4. 构建生产版本 (Build)
打包生成适用于你当前系统的安装包：
```bash
pnpm tauri build
```
构建完成后，安装包将位于 ```src-tauri/target/release/bundle/``` 目录下。
### 5. 其他命令
- 代码检查与格式化:
```bash
# 运行 ESLint
pnpm run lint-staged
# 运行 Knip 检查无用文件
pnpm knip
```
- 测试:
```bash
pnpm test
```
## ✨ 特色功能
- 🔎 支持 Bilibili 音乐/视频综合搜索与播放
- 🎼 支持登录 Bilibili 并获取收藏夹信息
- 🎧 高品质音频播放，优先拉取更高码率音频流（如无损 Flac，192K/Hi-Res）
- 🧩 轻量界面，内置深色主题，同时可自定义部分主题样式，细腻的滚动与动效体验
- 💿 系统托盘与最小化隐藏（Windows），便捷控制播放
- ♻️ 自动检测更新，始终保持最新体验

## 下载和使用
*(以下为原项目下载方式，本 Fork 版本的构建产物请自行编译)*
- 下载页面：[Github Release](https://github.com/wood3n/biu/releases/latest)
- 在 Releases 中选择与你系统和架构匹配的安装包；常见文件名示例：
  - Windows 安装包：`Biu-<version>-win-setup.exe` / `Biu-<version>-win-setup-arm64.exe`
  - Windows 免安装版：`Biu-<version>-win-portable-x64.exe` / `Biu-<version>-win-portable-arm64.exe`（portable，免安装）
  - macOS：`Biu-<version>-mac-x64.dmg` / `Biu-<version>-mac-arm64.dmg`（或对应的 `zip`）
  - Linux：`Biu-<version>-linux-*.AppImage` / `*.deb` / `*.rpm`（支持 `x64` / `arm64`）
  - `*.yml`、`*.blockmap` 为自动更新辅助文件，手动下载时无需关注。

- 系统要求（建议）
  - Windows 10 / 11（`x64` / `arm64`）
  - macOS 12+（`x64` / `arm64`）
  - 主流 Linux 发行版（`x64` / `arm64`）

- 使用注意
  - 部分音频清晰度与解析可能需要登录或大会员权限。
  - 请遵循 Bilibili 使用条款，合理合规使用。

## 🤝 贡献指南
非常欢迎社区贡献！你可以按以下流程参与：

1. Fork 本仓库并创建分支：`feature/your-feature` / `fix/your-fix`
2. 开发并通过本地构建与基本自测（如：`pnpm dev`、`pnpm build`）
3. 提交 PR，详述改动点与影响范围
4. 通过 CI 的构建与审查后合入主分支

建议：
- 保持代码风格一致（ESLint/Prettier 已配置）
- 提交信息简洁规范（推荐使用 `feat: ...`、`fix: ...` 等约定式格式）
- PR 中附上必要的截图或说明

## 📄 许可证
本项目以 PolyForm Noncommercial License 1.0.0（非商业许可）发布，禁止任何商业用途。详情参见 [`LICENSE`](LICENSE)（SPDX：`PolyForm-Noncommercial-1.0.0`）。

---

如果你喜欢这个项目，欢迎 ⭐️ Star 支持！也欢迎提出 Issue 交流与反馈 🙌

## 🙏 鸣谢
- 特别感谢 [SocialSisterYi/bilibili-API-collect](https://github.com/SocialSisterYi/bilibili-API-collect) 对哔哩哔哩 API 的长期收集与整理，为本项目相关接口的使用提供了重要参考。
- 在引用与使用相关资料时，我们遵循其许可条款（`CC-BY-NC 4.0`），仅用于学习与研究，不涉及任何商业用途。

## ⚖️ 法律声明与使用限制
- 本项目仅供学习与研究使用，禁止任何形式的商业用途（包括但不限于销售、收费服务、广告变现、商业集成等）。
- 本项目与 Bilibili 无任何官方关联或背书，不使用其商标与标识；涉及的名称与商标归其权利人所有。
- 数据来源于用户调用的公开接口与个人账户授权；使用时需遵守 Bilibili 的《用户协议》《社区规则》及相关法律法规。
- 禁止绕过登录/会员权限、DRM/加密措施，或进行批量爬取、恶意抓取等违反平台规则的行为。
- 如需商业授权或调整许可，请联系作者；如涉及权利或合规问题，请通过 Issues 反馈以便及时处理。
