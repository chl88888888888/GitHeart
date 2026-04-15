Git 心电图 (GitHeart)

Git 心电图 是一款Tauri桌面应用，通过分析 Git 仓库的提交历史，将代码变更频率可视化为“心电图”，帮助开发者识别代码热点、发现架构风险、洞察团队开发节奏。

---

功能特性

· 本地仓库分析 – 直接打开本地 Git 仓库，扫描最近 500 次提交
· GitHub 云端分析 – 输入 GitHub 仓库 URL，通过 API 获取提交数据（支持 Personal Access Token 提升速率限制）
· 动态心电图 – 按提交顺序动态绘制代码流失量（变更行数），并配有音效反馈，直观感受代码变动剧烈程度
· 月度提交趋势图 – 以折线图展示每月提交数量变化
· 代码热点表格 – 列出修改最频繁的文件，展示改动次数、增删行数、流失指数及风险评级
· 智能洞察 – 自动识别“上帝文件”、高流失文件、配置文件频繁变动、提交集中度异常等风险点

技术栈

桌面框架 Tauri v2
前端 Vue 3 + TypeScript + Vite
图表 ECharts (vue-echarts)
后端（Rust） git2 (本地仓库解析) + octocrab (GitHub API) + chrono + serde
构建工具 Cargo + npm

安装与运行

前置条件

· Rust 1.94.0 和 Cargo
· Node.js 和  npm
· 系统依赖：参考 Tauri 前置依赖

克隆与安装

```bash
git clone https://github.com/chl88888888888/GitHeart.git
cd githeart
```

安装前端依赖

```bash
npm install
```

开发模式运行

```bash
npm tauri dev
```

首次运行会自动下载 WebView2 运行时（Windows）或使用系统 webview（macOS/Linux）。

生产构建

```bash
npm tauri build
```

构建产物位于 src-tauri/target/release/bundle/。

使用指南

1. 分析本地仓库
      点击“选择本地 Git 仓库”，选择任意包含 .git 文件夹的目录。应用会读取最近 500 次提交并生成分析报告。
2. 分析 GitHub 仓库
   · 在输入框中填写 GitHub 仓库地址（支持 owner/repo、https://github.com/owner/repo 等格式）
   · 可选：添加 GitHub Personal Access Token（无需任何权限，仅提升 API 速率限制至 5000 次/小时）
   · 点击“分析云端仓库”，等待 API 数据拉取完成。
3. 解读结果
   · 动态心电图：按时间顺序播放每次提交的代码流失量（红线高度表示变更剧烈程度），可暂停/调速/拖拽。
   · 月度趋势：查看项目活跃度变化。
   · 热点文件：找出修改最频繁的文件，结合“流失指数”（(添加行+删除行)/修改次数）评估代码稳定性。
   · 智能洞察：直接获取架构改进建议。

配置

GitHub API 速率限制

· 无 Token：每小时 60 次请求（约可分析 3~5 个小仓库）
· 有 Token：每小时 5000 次请求（推荐使用）

Token 仅在本地存储，不会被上传。

限制说明

· 最多分析 500 次提交（超出部分自动截断）
· GitHub 分析依赖 octocrab，自动重试速率限制错误（最多 3 次）

项目结构

```
githeart/
├── src-tauri/               # Rust 后端
│   ├── src/
│   │   ├── commands/        # Tauri 命令实现
│   │   │   ├── git_analyzer.rs     # 本地仓库分析
│   │   │   └── github_analyzer.rs  # GitHub API 分析
│   │   ├── models.rs        # 数据结构 (AnalysisResult, FileStats 等)
│   │   └── lib.rs           # Tauri 应用入口
│   └── Cargo.toml
├── src/                     # Vue 前端
│   ├── components/          # UI 组件
│   │   ├── DynamicECG.vue   # 动态心电图画板
│   │   ├── ECGChart.vue     # 月度趋势图
│   │   ├── HotspotTable.vue # 热点文件表格
│   │   ├── InsightCard.vue  # 智能洞察卡片
│   │   ├── FolderPicker.vue # 本地文件夹选择
│   │   └── GithubInput.vue  # GitHub URL 输入
│   ├── composables/
│   │   └── useGitAnalyzer.ts # 封装 Tauri 调用
│   ├── App.vue
│   └── main.ts
└── package.json
```

致谢

· Tauri – 高性能桌面框架
· git2-rs – 本地 Git 仓库解析
· octocrab – GitHub API 客户端
· ECharts – 强大的图表库

---

Git 心电图 – 让你的代码健康“听”得见