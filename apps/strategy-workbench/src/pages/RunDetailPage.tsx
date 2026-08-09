import { Link, useParams } from "@tanstack/react-router";
import { ArrowLeft, LockKeyhole } from "lucide-react";

import type {
  BacktestEquityPoint,
  BacktestPosition,
  BacktestTrade,
} from "../api/generated/productApi";

import {
  environmentLabels,
  formatTimestamp,
  lifecycleLabels,
  resultLabels,
  riskLabels,
} from "../features/product/presentation";
import { useRunProductContext } from "../features/product/useProductResources";
import { ProductErrorState, ProductLoading } from "./ProductState";
import styles from "./Pages.module.css";

export function RunDetailPage() {
  const { runId } = useParams({ from: "/runs/$runId" });
  const product = useRunProductContext(runId);

  if (product.error) return <ProductErrorState error={product.error} />;
  if (product.isVerifying || !product.isReady || !product.run) {
    return <ProductLoading label="正在验证 Run 详情" />;
  }

  const detail = product.run;
  const pnlCurrency = product.metrics
    ? Object.keys(product.metrics.metrics.pnl_stats).sort()[0]
    : undefined;
  const pnlStats = pnlCurrency
    ? product.metrics?.metrics.pnl_stats[pnlCurrency]
    : undefined;
  const returnStats = product.metrics?.metrics.return_stats;
  return (
    <>
      <header className={styles.pageHeading}>
        <div>
          <Link to="/overview" className={styles.backLink}>
            <ArrowLeft aria-hidden="true" /> 返回策略总览
          </Link>
          <span className="eyebrow">Run 详情</span>
          <h1>{detail.run_id}</h1>
          <p>
            {product.strategy?.name ?? detail.strategy_id} ·{" "}
            {detail.strategy_version_id}
          </p>
        </div>
        <span className={styles.readOnlyBadge}>
          <LockKeyhole aria-hidden="true" /> 只读详情
        </span>
      </header>

      <section
        className={`${styles.connectionBanner} ${
          detail.risk.status === "blocked"
            ? styles.connectionBlocked
            : styles.connectionReady
        }`}
        aria-live="polite"
      >
        <div>
          <strong>
            {environmentLabels[detail.environment]} ·{" "}
            {lifecycleLabels[detail.lifecycle]}
          </strong>
          <span>
            风险 {riskLabels[detail.risk.status]} · 结果{" "}
            {resultLabels[detail.result.status]}
          </span>
        </div>
        <em>
          {detail.source.freshness_status === "fresh" ? "来源新鲜" : "来源阻断"}
        </em>
      </section>

      <section className={styles.metricGrid} aria-label="Run 摘要">
        <Metric
          label="环境"
          value={environmentLabels[detail.environment]}
          note={detail.adapter_ref}
        />
        <Metric
          label="生命周期"
          value={lifecycleLabels[detail.lifecycle]}
          note={formatTimestamp(detail.updated_at_unix_ms)}
        />
        <Metric
          label="风险"
          value={riskLabels[detail.risk.status]}
          note={detail.risk.risk_ref}
          warning={detail.risk.status === "blocked"}
        />
        <Metric
          label="结果"
          value={resultLabels[detail.result.status]}
          note={detail.result.result_ref ?? "尚无结果产物"}
        />
      </section>

      {product.metrics ? (
        <section className={styles.panel} aria-label="Backtest 指标">
          <header>
            <div>
              <span className="eyebrow">Backtest 指标</span>
              <h2>真实引擎回测结果</h2>
            </div>
            <span>研究结果，不代表 Live 准入</span>
          </header>
          <div className={styles.metricGrid}>
            <Metric
              label="订单"
              value={String(product.metrics.metrics.total_orders)}
              note={`${product.metrics.metrics.total_events} 个事件`}
            />
            <Metric
              label="持仓"
              value={String(product.metrics.metrics.total_positions)}
              note={`${product.metrics.metrics.iterations} 次迭代`}
            />
            <Metric
              label="行情样本"
              value={String(product.metrics.metrics.quotes)}
              note={product.metrics.data_ref}
            />
            <Metric
              label="回测区间"
              value={formatNanos(product.metrics.backtest_start)}
              note={`至 ${formatNanos(product.metrics.backtest_end)}`}
            />
          </div>
          <div className={styles.metricGrid} aria-label="Backtest 收益统计">
            <Metric
              label="总损益"
              value={displayStat(pnlStats?.["PnL (total)"])}
              note={pnlCurrency ?? "无结算币种"}
            />
            <Metric
              label="累计收益率"
              value={displayStat(pnlStats?.["PnL% (total)"])}
              note="引擎原始统计"
            />
            <Metric
              label="胜率"
              value={displayStat(pnlStats?.["Win Rate"])}
              note="已完成交易"
            />
            <Metric
              label="夏普比率"
              value={displayStat(returnStats?.["Sharpe Ratio (252 days)"])}
              note="252 天年化"
            />
            <Metric
              label="索提诺比率"
              value={displayStat(returnStats?.["Sortino Ratio (252 days)"])}
              note="252 天年化"
            />
            <Metric
              label="收益波动率"
              value={displayStat(
                returnStats?.["Returns Volatility (252 days)"],
              )}
              note={product.metrics.instrument_id}
            />
          </div>
        </section>
      ) : null}

      {product.report ? (
        <BacktestReport
          trades={product.report.trades}
          positions={product.report.positions}
          equity={product.report.equity_curve}
        />
      ) : detail.result.report_ref && product.isReportVerifying ? (
        <ProductLoading label="正在验证 Backtest 结果明细" />
      ) : detail.result.report_ref && product.reportError ? (
        <ProductErrorState
          error={product.reportError}
          onRetry={product.retryReport}
          retrying={product.isReportVerifying}
        />
      ) : product.metrics ? (
        <section className={styles.panel} aria-label="Backtest 结果明细">
          <header>
            <div>
              <span className="eyebrow">结果明细</span>
              <h2>历史 Run 仅保留聚合指标</h2>
            </div>
            <span>重新运行后生成明细</span>
          </header>
        </section>
      ) : null}

      <div className={styles.detailGrid}>
        <section className={styles.panel}>
          <header>
            <div>
              <span className="eyebrow">运行绑定</span>
              <h2>数据、账户与执行环境</h2>
            </div>
          </header>
          <div className={styles.versionSummary}>
            <KeyValue label="Strategy" value={detail.strategy_id} />
            <KeyValue
              label="StrategyVersion"
              value={detail.strategy_version_id}
            />
            <KeyValue
              label="内容 Hash"
              value={product.version?.content_hash ?? "未加载"}
              mono
            />
            <KeyValue label="数据" value={detail.data_ref} mono />
            <KeyValue label="配置" value={detail.config_ref} mono />
            <KeyValue label="适配器" value={detail.adapter_ref} mono />
            <KeyValue label="账户" value={detail.account_ref} mono />
            <KeyValue label="Venue" value={detail.venue_ref} mono />
          </div>
        </section>

        <section className={styles.panel}>
          <header>
            <div>
              <span className="eyebrow">能力边界</span>
              <h2>当前 Run 禁止能力</h2>
            </div>
            <span className={styles.warning}>全部关闭</span>
          </header>
          <div className={styles.boundaryGrid}>
            <Boundary
              label="外部 Venue 连接"
              enabled={detail.capabilities.external_venue_connection}
            />
            <Boundary
              label="订单提交"
              enabled={detail.capabilities.order_submission_allowed}
            />
            <Boundary
              label="订单修改"
              enabled={detail.capabilities.order_mutation_allowed}
            />
            <Boundary
              label="自动重试"
              enabled={detail.capabilities.automatic_retry_allowed}
            />
            <Boundary
              label="自动补救"
              enabled={detail.capabilities.automatic_remediation_allowed}
            />
            <Boundary
              label="真实订单"
              enabled={detail.capabilities.real_orders_submitted}
            />
            <Boundary
              label="交易控件"
              enabled={detail.capabilities.trading_controls_enabled}
            />
          </div>
        </section>

        <section className={`${styles.panel} ${styles.sourcePanel}`}>
          <header>
            <div>
              <span className="eyebrow">审计来源</span>
              <h2>可追溯引用</h2>
            </div>
            <span>{product.requestId}</span>
          </header>
          <div className={styles.sourceRows}>
            {detail.source.source_refs.map((source) => (
              <div key={source}>
                <span>来源</span>
                <strong>{source}</strong>
              </div>
            ))}
          </div>
        </section>
      </div>
    </>
  );
}

function BacktestReport({
  trades,
  positions,
  equity,
}: {
  trades: BacktestTrade[];
  positions: BacktestPosition[];
  equity: BacktestEquityPoint[];
}) {
  return (
    <div className={styles.reportGrid} aria-label="Backtest 结果明细">
      <section className={`${styles.panel} ${styles.reportWide}`}>
        <header>
          <div>
            <span className="eyebrow">账户权益</span>
            <h2>已实现收益曲线</h2>
          </div>
          <span>{equity[0]?.currency ?? "--"}</span>
        </header>
        <EquityCurve points={equity} />
      </section>

      <section className={styles.panel} aria-label="交易明细">
        <header>
          <div>
            <span className="eyebrow">成交记录</span>
            <h2>交易明细</h2>
          </div>
          <span>{trades.length} 笔</span>
        </header>
        <div className={styles.tableWrap}>
          <table className={styles.detailTable}>
            <thead>
              <tr>
                <th>Trade</th>
                <th>时间</th>
                <th>方向</th>
                <th>数量</th>
                <th>价格</th>
                <th>手续费</th>
              </tr>
            </thead>
            <tbody>
              {trades.map((trade) => (
                <tr key={trade.trade_id}>
                  <td>{trade.trade_id}</td>
                  <td>{formatNanos(trade.ts_event)}</td>
                  <td className={sideClass(trade.side)}>{trade.side}</td>
                  <td>{trade.quantity}</td>
                  <td>{trade.price}</td>
                  <td>{trade.commission ?? "--"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className={styles.panel} aria-label="持仓明细">
        <header>
          <div>
            <span className="eyebrow">持仓周期</span>
            <h2>持仓明细</h2>
          </div>
          <span>{positions.length} 个</span>
        </header>
        <div className={styles.tableWrap}>
          <table className={styles.detailTable}>
            <thead>
              <tr>
                <th>Position</th>
                <th>方向</th>
                <th>开仓均价</th>
                <th>平仓均价</th>
                <th>已实现损益</th>
              </tr>
            </thead>
            <tbody>
              {positions.map((position) => (
                <tr key={position.position_id}>
                  <td>{position.position_id}</td>
                  <td className={sideClass(position.entry_side)}>
                    {position.entry_side}
                  </td>
                  <td>{position.avg_price_open}</td>
                  <td>{position.avg_price_close ?? "--"}</td>
                  <td>{position.realized_pnl ?? "未实现"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}

function EquityCurve({ points }: { points: BacktestEquityPoint[] }) {
  const totals = points.map((point) => moneyValue(point.total));
  const minimum = Math.min(...totals);
  const maximum = Math.max(...totals);
  const range = maximum - minimum || 1;
  const chartPoints = totals
    .map((value, index) => {
      const x =
        points.length === 1 ? 300 : (index / (points.length - 1)) * 580 + 10;
      const y = 105 - ((value - minimum) / range) * 90;
      return `${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(" ");
  const first = points[0];
  const last = points.at(-1);

  return (
    <div className={styles.equityChart}>
      <div>
        <span>起始权益</span>
        <strong>{first?.total ?? "--"}</strong>
      </div>
      <svg
        viewBox="0 0 600 120"
        preserveAspectRatio="none"
        role="img"
        aria-label="账户权益随回测时间变化"
      >
        <line x1="10" y1="105" x2="590" y2="105" />
        <polyline points={chartPoints} />
      </svg>
      <div>
        <span>结束权益</span>
        <strong>{last?.total ?? "--"}</strong>
      </div>
    </div>
  );
}

function moneyValue(value: string): number {
  const parsed = Number(value.split(" ")[0]?.replaceAll("_", ""));
  return Number.isFinite(parsed) ? parsed : 0;
}

function sideClass(side: string): string {
  return side === "BUY" ? styles.sideBuy : styles.sideSell;
}

function formatNanos(value: string): string {
  const millis = Number(BigInt(value) / 1_000_000n);
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    timeZone: "UTC",
  }).format(new Date(millis));
}

function displayStat(value: string | undefined): string {
  if (!value) return "暂无数据";
  const parsed = Number(value);
  return Number.isFinite(parsed) ? value : "不可计算";
}

function Metric({
  label,
  value,
  note,
  warning,
}: {
  label: string;
  value: string;
  note: string;
  warning?: boolean;
}) {
  return (
    <article>
      <span>{label}</span>
      <strong className={warning ? styles.warning : ""}>{value}</strong>
      <small>{note}</small>
    </article>
  );
}

function KeyValue({
  label,
  value,
  mono,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className={styles.detailRow}>
      <span>{label}</span>
      <strong className={mono ? styles.mono : ""}>{value}</strong>
    </div>
  );
}

function Boundary({ label, enabled }: { label: string; enabled: boolean }) {
  return (
    <div className={styles.boundaryItem}>
      <span>{label}</span>
      <strong className={enabled ? styles.warning : ""}>
        {enabled ? "异常开启" : "关闭"}
      </strong>
    </div>
  );
}
