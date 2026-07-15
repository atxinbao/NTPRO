# Legacy Python Guides Retired

Date: 2026-07-15
Executor: Codex
Task ID: DEXG-004
Baseline: `ntpro-rust-only-v0.32.0`

## Decision

Five Python-first public pages were retired after the backend freeze because
the repository did not contain equivalent, authorized Rust workflows:

- `tutorials/fx_mean_reversion_ax.md`;
- `tutorials/gold_book_imbalance_ax.md`;
- `tutorials/grid_market_maker_bitmex.md`;
- `how_to/configure_live_trading.md`;
- `developer_guide/python.md`.

Plain Chinese summary: 这 5 页虽然标注为 legacy，但仍出现在当前导航中并包含可执行
Python 代码，和 Rust-only 文档入口冲突。仓库没有等价且已授权的 Rust 实现，因此本次
删除页面而不是伪造迁移；Git 历史保留原内容，当前导航改到 Rust example、Rust guide
或产品 contract。

## Asset Cleanup

The three removed tutorials were the only references to 12 PNG files under
their matching asset directories. Those files were removed after a
repository-wide reference check. The other 20 tutorial images remain tracked
and referenced by retained pages.

## Replacement Routes

- bounded live-node startup: `examples/rust/live/README.md`;
- live/sandbox command boundary:
  `docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md`;
- Rust workflow guides: `docs/how_to/index.md`;
- current Rust tutorials: `docs/tutorials/index.md`;
- Rust development conventions: `docs/developer_guide/rust.md`.

The remaining Rust live guide describes library construction. It is not
production go-live authorization. The v0.32.0 backend boundary continues to
forbid inherited production submit, mutation, adapter send, live exchange,
retry, remediation, and trading-control capabilities.
