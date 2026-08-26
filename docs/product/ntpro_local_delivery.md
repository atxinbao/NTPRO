# NTPRO 本地产品操作说明

日期：2026-08-26

执行者：Codex

## 这套交付是什么

NTPRO Usable Product v1 是一套在一台电脑上运行的策略工作台。普通用户拿到完整交付目录后，
只需要运行一个文件，不需要安装 Cargo、理解 Rust crate，也不需要分别启动 Supervisor、node、
Product API 和网页前端。

启动后，NTPRO 会完成以下工作：

1. 初始化一个独立、可持续保留的用户工作区；
2. 准备一个本地 Supervisor 和一个 Sandbox node；
3. 启动 Product API；
4. 提供 production 版本的策略工作台；
5. 在退出时停止仍在运行的 Demo node，并保留 Backtest、Demo 和状态数据。

Usable Product v1 的交付范围是 **Backtest + Demo**。Live 仍使用独立准入，不会因为启动本地
产品而自动连接真实 Venue、提交真实订单或取得订单权限。

## 你会收到什么

完整交付目录至少包含：

```text
ntpro-local-delivery/
├── start-ntpro                 # 唯一用户启动入口
├── 操作说明.md                 # 本说明
├── delivery-manifest.json      # 交付来源和关键文件摘要
├── bin/                        # NTPRO 主程序和 node
├── configs/                    # 已验证的单策略配置
└── apps/strategy-workbench/    # 已构建的 production 网页
```

不要只移动 `start-ntpro`；整个目录必须保持完整。

## 第一次启动

打开终端，进入交付目录，然后运行：

```bash
./start-ntpro
```

终端会显示工作区、固定访问地址和带一次性本地访问凭据的策略工作台地址。等待看到
`strategy_workbench_url=` 后，在本机浏览器打开该完整地址。浏览器会自动清除地址中的临时凭据，
随后进入 `/strategy-workbench/overview`。

默认访问地址：

```text
http://127.0.0.1:5173/strategy-workbench/overview
```

第一次直接输入默认地址可能没有访问凭据，因此应先使用终端本次打印的完整地址。服务重启后凭据
会轮换，旧浏览器会话被拒绝时，重新使用新终端输出即可。

## 在页面里完成 Backtest

1. 左侧进入“回测”；
2. 选择已经验证的数据来源；
3. 检查策略版本、参数、品种和数据范围；
4. 显式点击创建回测；
5. 在 Run 详情查看指标、交易、持仓、资金曲线、风险、日志和来源；
6. 需要时进入比较页选择多个 Backtest Run。

创建操作不会自动重试。页面显示来源未验证、陈旧或错误时，不要把空值当作真实结果；按页面提示
刷新或检查系统状态后，再由用户显式重试。

## 在页面里完成 Demo

1. 左侧进入“Demo”；
2. 确认策略身份和节点状态均已验证；
3. 勾选确认后创建 Demo Run；
4. 点击“启动”，观察 Sandbox 策略状态、模拟成交、持仓和权益；
5. 点击“停止”，等待终态结果和结果哈希冻结；
6. 刷新页面或重启 NTPRO 后，可继续读取已停止的 Demo Run。

Demo 使用同一 StrategyVersion，但不会继承 Live 凭据或真实交易权限。已经停止的 Demo 在 NTPRO
重启后不会自动恢复运行。

## 正常停止和再次启动

回到启动 NTPRO 的终端，按一次 `Ctrl-C`。看到以下提示后才算停止完成：

```text
NTPRO 已安全停止。
```

不要直接删除交付目录或工作区来停止服务。再次运行 `./start-ntpro` 会使用同一工作区，历史
Backtest、Demo 和终态结果继续保留，但本地访问凭据会重新生成。

默认数据位置：

- macOS：`~/Library/Application Support/NTPRO/usable-product-v1`
- Linux：`${XDG_DATA_HOME:-~/.local/share}/ntpro/usable-product-v1`

删除或移动这个工作区会影响历史数据。升级交付目录时，不要覆盖用户工作区。

## 状态、数据和日志在哪里

日常使用优先查看策略工作台的“系统状态”和 Run 详情，不需要手工查看文件。

需要排障时，以下路径位于上面的用户工作区：

| 内容 | 路径 |
| --- | --- |
| Supervisor 注册表 | `supervisor/registry.json` |
| MVP 身份与状态 | `mvp/identity_contract.json`、`mvp/status_contract.json` |
| Backtest 工件 | `artifacts/backtests/` |
| Demo 工件 | `artifacts/demo-runs/` |
| node 状态和指标 | `nodes/mvp-node-001/` |
| node 标准输出 | `nodes/mvp-node-001/logs/stdout.log` |
| node 错误输出 | `nodes/mvp-node-001/logs/stderr.log` |
| node 事件日志 | `nodes/mvp-node-001/logs/events.log` |

启动器自身的错误直接显示在当前终端。终端可能包含一次性本地访问地址，不要把完整输出公开传播。

## 常见问题

### 提示“已经有一个 NTPRO 实例在运行”

不要重复启动。回到原终端继续使用，或者先按 `Ctrl-C` 正常停止原实例。启动器会检查实际 PID，
不会只凭一个遗留文件判断系统仍在运行。

### 提示端口已被占用

关闭占用 `5173` 的程序，或临时选择另一个本机端口：

```bash
NTPRO_BIND=127.0.0.1:5180 ./start-ntpro
```

随后使用终端打印的新地址。不要绑定公网地址。

### 上次异常退出后无法启动

直接再次运行 `./start-ntpro`。如果旧进程已不存在，启动器会识别并清理失效运行锁。若仍提示实例
在运行，先确认原终端或原进程是否仍然存在，不要手工删除运行中的工作区。

### 提示缺少主程序、node、配置或策略工作台

交付目录不完整。不要自行从源码目录拼装文件，应重新获取完整交付目录并核对
`delivery-manifest.json`。

### 页面打不开或显示未验证

1. 确认启动终端仍在运行；
2. 使用本次启动输出的完整 `strategy_workbench_url`；
3. 查看页面“系统状态”；
4. 查看终端错误和 node 日志；
5. 正常停止后重新启动，不要连续重复点击页面操作。

HTTP 端口可访问只说明网页服务能够响应，不代表 node、策略状态或数据来源已经健康。以页面四轴状态
和具体 Run 状态为准。

## 交付构建者说明

本节只面向从源码生成交付目录的开发者，不是普通用户操作步骤。

在仓库根目录运行：

```bash
scripts/ai/build_ntpro_local_delivery.sh
```

默认输出到 `target/ntpro-local-delivery`。构建器会编译两个 Rust 二进制、构建 Vite production
bundle、复制必要配置和说明，并生成 `delivery-manifest.json`。已经准备好二进制和前端产物时，
自动验收可使用：

```bash
NTPRO_LOCAL_DELIVERY_SKIP_BUILD=1 scripts/ai/build_ntpro_local_delivery.sh
```

生成目录是可重建交付物；用户工作区是持久数据，两者必须分开备份和升级。
