# CPA Monitor

实时监控 LLM API 用量事件的终端仪表盘工具。

## 概述

CPA Monitor 是一个 Rust 工作区项目，用于消费、存储和展示 LLM API 调用的用量数据（Token 消耗、延迟、失败率等）。系统采用 **Redis 消费 → PostgreSQL 存储 → TUI 仪表盘展示** 的管道架构。

![](docs/demo.png)

## 架构

项目包含 4 个 crate：

| crate | 用途 |
|-------|------|
| **cpa-ingestor** | 从 Redis Pub/Sub 频道消费用量事件 JSON，解析后写入 PostgreSQL |
| **cpa-aggregator** | 基于 ratatui 的终端仪表盘，实时展示用量统计、趋势图和最近请求 |
| **cpa-config** | 配置加载（TOML 文件 + 环境变量覆盖） |
| **cpa-store** | 数据库模型、Schema 定义、查询逻辑（基于 sea-orm） |

### 数据流

```
LLM API 代理 → Redis Pub/Sub → cpa-ingestor → PostgreSQL → cpa-aggregator (TUI)
```

## 快速开始

### 前置要求

- Rust 1.94+
- PostgreSQL
- Redis

### 配置文件

默认配置路径：`~/.config/cpa-monitor/config.toml`，可通过 `CPA_CONFIG_DIR` 环境变量覆盖。

```toml
[database]
url = "postgres://postgres:postgres@localhost:5432/cpa_monitor"

[redis]
url = "redis://127.0.0.1:6379"
channel = "usage"
```

也可以通过环境变量配置：

```bash
export CPA__DATABASE__URL="postgres://user:pass@host:5432/cpa_monitor"
export CPA__REDIS__URL="redis://127.0.0.1:6379"
export CPA__REDIS__CHANNEL="usage"
```

### 启动

```bash
# 1. 启动数据消费
cargo run --bin cpa-ingestor

# 2. 启动仪表盘（另一个终端）
cargo run --bin cpa-aggregator
```

### 用量事件格式

向 Redis 频道发送如下 JSON 格式的消息：

```json
{
  "timestamp": "2026-04-25T00:00:00Z",
  "latency_ms": 1500,
  "source": "user@example.com",
  "auth_index": "0",
  "tokens": {
    "input_tokens": 100,
    "output_tokens": 200,
    "reasoning_tokens": 0,
    "cached_tokens": 50,
    "total_tokens": 300
  },
  "failed": false,
  "provider": "openai",
  "model": "gpt-5.4",
  "alias": "client-gpt",
  "endpoint": "POST /v1/chat/completions",
  "auth_type": "apikey",
  "api_key": "sk-...",
  "request_id": "req-xxx"
}
```

## 仪表盘功能

- **用量摘要** — 24h Token 总数、请求数、失败率、平均/P95 延迟
- **分组统计** — 按 Model 和 Provider 分组的用量与失败数
- **趋势图** — 24 小时小时级 Token 趋势 + 近 5 分钟实时迷你趋势
- **最近请求** — 最新请求列表，含状态、延迟等详情
- **交互** — `r` 手动刷新，`q`/`Esc` 退出，每 3 秒自动刷新

## 项目结构

```
cpa-monitor/
├── config/              # 默认 TOML 配置文件
├── crates/
│   ├── cpa-ingestor/    # Redis 消费 → DB 写入
│   ├── cpa-aggregator/  # TUI 仪表盘
│   ├── cpa-config/      # 配置管理
│   └── cpa-store/       # 数据模型与查询
└── Cargo.toml           # 工作区定义
```

## 技术栈

- **Rust 2024 edition**
- **tokio** — 异步运行时
- **sea-orm** — ORM / PostgreSQL
- **redis** — Redis 客户端
- **ratatui + crossterm** — TUI 框架
- **serde / chrono** — 序列化与时间处理
