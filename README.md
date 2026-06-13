# CPA Monitor

实时监控 LLM API 用量事件的终端仪表盘工具。

## 概述

CPA Monitor 是一个 Rust 工作区项目，用于消费、存储和展示 LLM API 调用的用量数据（Token 消耗、延迟、失败率等）。系统采用 **Redis Pub/Sub 消费 → PostgreSQL 存储 → TUI 仪表盘展示** 的管道架构。

## 架构

### 组件

项目包含 4 个 crate：

| crate | 用途 |
|-------|------|
| **cpa-ingestor** | 从 Redis Pub/Sub 频道消费用量事件 JSON，解析后写入 PostgreSQL |
| **cpa-aggregator** | 基于 ratatui 的终端仪表盘，实时展示用量统计、趋势图和最近请求 |
| **cpa-config** | 配置加载（TOML 文件 + 环境变量覆盖） |
| **cpa-store** | 数据库模型、Schema DDL、仪表盘聚合查询（基于 sea-orm） |

### 数据流

```
LLM API 代理 → Redis Pub/Sub → cpa-ingestor → PostgreSQL → cpa-aggregator (TUI)
```

### 数据模型

`usage_events` 表存储每次 API 调用的完整信息：

- 时间戳、来源、请求 ID
- Token 明细（input / output / reasoning / cached / total）
- 延迟（毫秒）与失败状态
- Provider、Model、Alias、Endpoint
- 认证信息（auth_type、auth_index、api_key）

### 仪表盘

仪表盘每 3 秒自动刷新，展示 24 小时滚动窗口的数据：

- **摘要面板** — 总 Token、输入/输出/推理/缓存 Token、请求数、失败率、平均延迟、P95 延迟
- **分组统计** — 按 Model 和 Provider 分组的 Token 用量和请求数（Top 8）
- **趋势图** — 24 小时小时级 Token 趋势 + 近 5 分钟 10 秒桶实时迷你趋势
- **最近请求** — 最近 12 条请求详情（时间、模型、Provider、Token、延迟、状态）
- **交互** — `r` 手动刷新，`q`/`Esc` 退出

## 快速开始

### 前置要求

- Rust 1.94+
- PostgreSQL（运行中）
- Redis（运行中）

### 配置文件

默认配置路径：`~/.config/cpa-monitor/default.toml`，可通过 `CPA_CONFIG_DIR` 环境变量覆盖。

```toml
[database]
url = "postgres://postgres:postgres@localhost:5432/cpa_monitor"

[redis]
url = "redis://127.0.0.1:6379"
channel = "usage"
```

也可以通过环境变量覆盖所有配置（优先级高于配置文件）：

```bash
export CPA__DATABASE__URL="postgres://user:pass@host:5432/cpa_monitor"
export CPA__REDIS__URL="redis://127.0.0.1:6379"
export CPA__REDIS__CHANNEL="usage"
```

### 启动

```bash
# 终端 1：启动数据消费（从 Redis 订阅并写入 PostgreSQL）
cargo run --bin cpa-ingestor

# 终端 2：启动 TUI 仪表盘
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

事件中的 `response_headers` 等额外字段会被忽略，不影响解析。

### Docker 部署

仅部署 ingestor 到已有 PostgreSQL 和 Redis 容器所在的 Docker 网络：

```bash
docker network create cpa-net
docker network connect cpa-net postgres
docker network connect cpa-net redis
docker compose up -d
```

## 项目结构

```
cpa-monitor/
├── config/                  # TOML 配置文件
│   └── default.toml
├── crates/
│   ├── cpa-ingestor/        # Redis 消费 → DB 写入
│   │   └── src/main.rs
│   ├── cpa-aggregator/      # TUI 仪表盘（ratatui）
│   │   └── src/main.rs
│   ├── cpa-config/          # 配置管理
│   │   └── src/lib.rs
│   └── cpa-store/           # 数据模型与查询
│       ├── src/
│       │   ├── lib.rs        # DB 连接、Schema 初始化、事件写入
│       │   ├── usage_event.rs# sea-orm 实体定义
│       │   └── dashboard.rs  # 聚合查询（摘要、分组、趋势、最近请求）
├── openspec/                # OpenSpec 规范文档
│   ├── config.yaml
│   └── specs/token-usage-dashboard/
├── Cargo.toml               # 工作区定义
├── Dockerfile               # cpa-ingestor 容器镜像
└── docker-compose.yml       # Docker Compose 部署
```

## 技术栈

- **Rust 2024 edition**
- **tokio** — 异步运行时
- **sea-orm + sqlx-postgres** — ORM / PostgreSQL
- **redis** — Redis 异步客户端（tokio-comp）
- **ratatui + crossterm** — TUI 框架
- **serde + chrono** — 序列化与时间处理
- **config** — 分层配置（TOML 文件 + 环境变量）
