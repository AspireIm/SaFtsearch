# SaFtsearch
[English](README.md) | **中文**
SaFtsearch 是一款正在制作中的桌面端极速文件搜索软件，计划使用 Rust 与 Python 共同实现。

> 项目状态：仍在开发中。当前仓库主要包含基础工程框架、早期 Rust 命令行检索逻辑，以及学习与实现文档。后续 API、命令、配置字段和桌面端接入方式都可能继续调整。

## 项目目标

- 面向桌面环境提供快速的本地文件搜索能力。
- Rust 负责扫描、索引、查询评分等性能敏感部分。
- Python 负责桌面 UI、配置管理、进程调度和用户交互。
- 搜索内核保持独立，方便 CLI 和 GUI 共同复用。

## 当前能力

- Rust workspace 和 search-core crate 已搭建。
- 已定义基础文件特征量模型。
- 支持目录扫描和排除规则。
- 支持简单文件名搜索与评分。
- 支持扫描结果和搜索结果的 JSON 输出。
- Python 应用入口目前是占位框架。

## 快速运行

检查 Rust 代码：

```powershell
cargo check
cargo test
```

输出默认配置：

```powershell
cargo run --bin saftsearch-indexer -- config
```

扫描当前项目：

```powershell
cargo run --bin saftsearch-indexer -- scan . --exclude target --exclude .git
```

按文件名搜索：

```powershell
cargo run --bin saftsearch-indexer -- search toml . --limit 10 --exclude target --exclude .git
```

运行 Python 应用占位入口：

```powershell
cd python-app\src
python -m saftsearch_app.main
```

## 后续计划

1. 先完成最小可运行的 Rust CLI 扫描与搜索。
2. 再让 Python 桌面层调用 Rust 搜索进程。
3. 引入持久化索引，避免每次查询都重新扫描。
4. 加入文件系统监听，实现增量更新。
5. 优化排序、过滤和桌面端交互。
6. 在文件名搜索稳定后，再扩展全文索引。

## 说明

本项目目前还不是生产可用版本。当前代码主要用于让整体架构先跑起来，并为后续逐步实现搜索内核、索引存储和桌面 UI 打好基础。
