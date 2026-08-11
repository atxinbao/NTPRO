import { Link, useParams } from "@tanstack/react-router";
import { ArrowLeft, LockKeyhole, Play, Square } from "lucide-react";
import { useState } from "react";

import type {
  BacktestAnalysis,
  BacktestEquityPoint,
  BacktestReproductionProof,
} from "../api/generated/productApi";

import {
  environmentLabels,
  formatTimestamp,
  lifecycleLabels,
  resultLabels,
  riskLabels,
} from "../features/product/presentation";
import {
  useDemoRunAction,
  useRunProductContext,
} from "../features/product/useProductResources";
import { ProductErrorState, ProductLoading } from "./ProductState";
import styles from "./Pages.module.css";

export function RunDetailPage() {
  const { runId } = useParams({ from: "/runs/$runId" });
  const product = useRunProductContext(runId);
  const demoAction = useDemoRunAction();
  const [actionError, setActionError] = useState<string>();

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
          {detail.environment === "sandbox" ? (
            <Play aria-hidden="true" />
          ) : (
            <LockKeyhole aria-hidden="true" />
          )}
          {detail.environment === "sandbox" ? "Sandbox 生命周期" : "只读详情"}
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

      {detail.environment === "sandbox" && detail.runtime ? (
        <section className={styles.panel} aria-label="Demo 生命周期">
          <header>
            <div>
              <span className="eyebrow">Supervisor</span>
              <h2>Demo 节点生命周期</h2>
            </div>
            <span>{detail.runtime.supervisor_node_id}</span>
          </header>
          <div className={styles.versionSummary}>
            <KeyValue label="节点进程" value={detail.runtime.process_state} />
            <KeyValue label="运行状态" value={detail.runtime.lifecycle_state} />
            <KeyValue
              label="策略实例"
              value={detail.runtime.strategy_instance_id}
              mono
            />
            <KeyValue
              label="策略版本"
              value={detail.strategy_version_id}
              mono
            />
          </div>
          {actionError ? (
            <div className={styles.formError} role="alert">
              {actionError}
            </div>
          ) : null}
          <div className={styles.runActions}>
            <span>每次动作都由用户显式触发，客户端不会自动重试。</span>
            <button
              type="button"
              disabled={demoAction.isPending || detail.lifecycle !== "created"}
              onClick={() => {
                setActionError(undefined);
                demoAction.mutate(
                  { runId: detail.run_id, action: "start" },
                  { onError: (error) => setActionError(error.message) },
                );
              }}
            >
              <Play aria-hidden="true" /> 启动
            </button>
            <button
              type="button"
              disabled={
                demoAction.isPending ||
                !["running", "paused"].includes(detail.lifecycle)
              }
              onClick={() => {
                setActionError(undefined);
                demoAction.mutate(
                  { runId: detail.run_id, action: "stop" },
                  { onError: (error) => setActionError(error.message) },
                );
              }}
            >
              <Square aria-hidden="true" /> 停止
            </button>
          </div>
        </section>
      ) : null}

      {product.demoSnapshot ? (
        <section className={styles.panel} aria-label="Demo 运行结果">
          <header>
            <div>
              <span className="eyebrow">Demo 运行结果</span>
              <h2>
                {product.demoSnapshot.snapshot_status === "frozen"
                  ? "终态冻结快照"
                  : product.demoSnapshot.snapshot_status === "running"
                    ? "实时策略快照"
                    : "等待启动"}
              </h2>
            </div>
            <span>
              {product.demoSnapshot.technical_health.status === "healthy"
                ? "运行健康"
                : "当前阻断"}
            </span>
          </header>
          <div className={styles.metricGrid}>
            <Metric
              label="行情事件"
              value={String(
                product.demoSnapshot.session?.market_event_count ?? 0,
              )}
              note={product.demoSnapshot.market?.state ?? "尚未启动"}
            />
            <Metric
              label="策略信号"
              value={String(product.demoSnapshot.session?.signal_count ?? 0)}
              note={product.demoSnapshot.latest_signal?.signal ?? "暂无信号"}
            />
            <Metric
              label="订单意图"
              value={String(product.demoSnapshot.session?.intent_count ?? 0)}
              note={
                product.demoSnapshot.latest_order_intent?.submission_status ??
                "暂无意图"
              }
            />
            <Metric
              label="风控拒绝"
              value={String(product.demoSnapshot.session?.rejection_count ?? 0)}
              note={
                product.demoSnapshot.latest_risk_decision?.decision ??
                "暂无决策"
              }
              warning={Boolean(product.demoSnapshot.session?.rejection_count)}
            />
            <Metric
              label="模拟成交"
              value={String(product.demoSnapshot.simulation?.fills.length ?? 0)}
              note="仅限 Sandbox，不会发送真实订单"
            />
            <Metric
              label="模拟持仓"
              value={String(
                product.demoSnapshot.simulation?.positions.length ?? 0,
              )}
              note={
                product.demoSnapshot.simulation?.summary.instrument_id ??
                "等待启动"
              }
            />
          </div>
          <div className={styles.versionSummary}>
            <KeyValue
              label="数据连接"
              value={product.demoSnapshot.runtime.data_connection}
            />
            <KeyValue
              label="执行连接"
              value={product.demoSnapshot.runtime.execution_connection}
            />
            <KeyValue
              label="最新行情"
              value={
                product.demoSnapshot.market?.latest_event
                  ? `${product.demoSnapshot.market.latest_event.symbol} · ${product.demoSnapshot.market.latest_event.price}`
                  : "暂无行情"
              }
            />
            <KeyValue
              label="最新风控原因"
              value={
                product.demoSnapshot.latest_risk_decision?.reasons.join("、") ||
                product.demoSnapshot.technical_health.diagnostics.join("、") ||
                "无"
              }
            />
            <KeyValue
              label="结果来源"
              value={
                product.demoSnapshot.provenance.result_ref ??
                product.demoSnapshot.provenance.source_refs[0]
              }
              mono
            />
            <KeyValue
              label="结果哈希"
              value={product.demoSnapshot.provenance.result_sha256 ?? "运行中"}
              mono
            />
          </div>
        </section>
      ) : null}

      {product.demoSnapshot?.simulation ? (
        <BacktestReport
          mode="Demo 模拟"
          trades={product.demoSnapshot.simulation.fills}
          positions={product.demoSnapshot.simulation.positions}
          equity={product.demoSnapshot.simulation.equity_curve}
        />
      ) : null}

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

      {product.analysis ? (
        <BacktestAnalysisPanel analysis={product.analysis} />
      ) : detail.result.analysis_ref && product.isAnalysisVerifying ? (
        <ProductLoading label="正在验证 Backtest 风险与运行记录" />
      ) : detail.result.analysis_ref && product.analysisError ? (
        <ProductErrorState
          error={product.analysisError}
          onRetry={product.retryAnalysis}
          retrying={product.isAnalysisVerifying}
          retryLabel="重试分析"
        />
      ) : product.metrics ? (
        <section className={styles.panel} aria-label="Backtest 分析">
          <header>
            <div>
              <span className="eyebrow">风险与运行记录</span>
              <h2>历史 Run 未生成分析产物</h2>
            </div>
            <span>重新运行后生成分析</span>
          </header>
        </section>
      ) : null}

      {product.reproduction ? (
        <ReproductionProof proof={product.reproduction} />
      ) : detail.result.reproduction_ref && product.isReproductionVerifying ? (
        <ProductLoading label="正在验证确定性复现证明" />
      ) : detail.result.reproduction_ref && product.reproductionError ? (
        <ProductErrorState
          error={product.reproductionError}
          onRetry={product.retryReproduction}
          retrying={product.isReproductionVerifying}
          retryLabel="重试复现证明"
        />
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

function ReproductionProof({ proof }: { proof: BacktestReproductionProof }) {
  return (
    <section className={styles.panel} aria-label="Backtest 确定性复现证明">
      <header>
        <div>
          <span className="eyebrow">确定性复现</span>
          <h2>输入与输出均等价</h2>
        </div>
        <span>{proof.user_initiated ? "用户主动创建" : "来源无效"}</span>
      </header>
      <div className={styles.versionSummary}>
        <KeyValue label="源 Run" value={proof.source_run_id} mono />
        <KeyValue label="复现 Run" value={proof.reproduced_run_id} mono />
        <KeyValue label="输入指纹" value={proof.source_input_sha256} mono />
        <KeyValue label="复现输入" value={proof.reproduced_input_sha256} mono />
        <KeyValue label="输出指纹" value={proof.source_output_sha256} mono />
        <KeyValue
          label="复现输出"
          value={proof.reproduced_output_sha256}
          mono
        />
        <KeyValue label="证明文件" value={proof.proof_ref} mono />
      </div>
    </section>
  );
}

function BacktestAnalysisPanel({ analysis }: { analysis: BacktestAnalysis }) {
  return (
    <div className={styles.reportGrid} aria-label="Backtest 风险与运行记录">
      <section
        className={`${styles.panel} ${styles.reportWide}`}
        aria-label="Backtest 资金曲线"
      >
        <header>
          <div>
            <span className="eyebrow">风险与回撤</span>
            <h2>账户权益回撤</h2>
          </div>
          <span>{analysis.risk.currency}</span>
        </header>
        <div className={styles.metricGrid}>
          <Metric
            label="最大回撤"
            value={formatRate(analysis.risk.max_drawdown_rate)}
            note={analysis.risk.max_drawdown_amount}
            warning={Number(analysis.risk.max_drawdown_rate) > 0}
          />
          <Metric
            label="当前回撤"
            value={formatRate(analysis.risk.current_drawdown_rate)}
            note={analysis.risk.current_drawdown_amount}
            warning={Number(analysis.risk.current_drawdown_rate) > 0}
          />
          <Metric
            label="峰值权益"
            value={analysis.risk.peak_equity}
            note={`结束 ${analysis.risk.ending_equity}`}
          />
          <Metric
            label="已平仓"
            value={String(analysis.risk.closed_positions)}
            note={`${analysis.risk.profitable_positions} 盈利 · ${analysis.risk.losing_positions} 亏损`}
          />
        </div>
        <DrawdownCurve analysis={analysis} />
      </section>

      <section className={styles.panel} aria-label="Backtest 运行记录">
        <header>
          <div>
            <span className="eyebrow">运行记录</span>
            <h2>结构化事件时间线</h2>
          </div>
          <span>{analysis.timeline.length} 条</span>
        </header>
        <div className={styles.tableWrap}>
          <table className={styles.detailTable}>
            <thead>
              <tr>
                <th>事件</th>
                <th>时间</th>
                <th>类型</th>
                <th>关联对象</th>
              </tr>
            </thead>
            <tbody>
              {analysis.timeline.map((event) => (
                <tr key={event.event_id}>
                  <td>{event.event_id}</td>
                  <td>{formatNanos(event.ts_event)}</td>
                  <td>{eventTypeLabel(event.event_type)}</td>
                  <td>{event.entity_ref}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className={styles.panel} aria-label="Backtest 分析来源">
        <header>
          <div>
            <span className="eyebrow">分析来源</span>
            <h2>不可变产物链</h2>
          </div>
          <span>{analysis.provenance.engine_mode}</span>
        </header>
        <div className={styles.sourceRows}>
          <SourceHash
            label="数据"
            reference={analysis.provenance.data_ref}
            hash={analysis.provenance.data_sha256}
          />
          <SourceHash
            label="配置"
            reference={analysis.provenance.config_ref}
            hash={analysis.provenance.config_sha256}
          />
          <SourceHash
            label="汇总"
            reference={analysis.provenance.summary_ref}
            hash={analysis.provenance.summary_sha256}
          />
          <SourceHash
            label="明细"
            reference={analysis.provenance.details_ref}
            hash={analysis.provenance.details_sha256}
          />
        </div>
      </section>
    </div>
  );
}

function DrawdownCurve({ analysis }: { analysis: BacktestAnalysis }) {
  const rates = analysis.drawdown_curve.map((point) =>
    Number(point.drawdown_rate),
  );
  const maximum = Math.max(...rates, 0);
  const range = maximum || 1;
  const points = rates
    .map((rate, index) => {
      const x =
        rates.length === 1 ? 300 : (index / (rates.length - 1)) * 580 + 10;
      const y = 15 + (rate / range) * 90;
      return `${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(" ");
  return (
    <div className={`${styles.equityChart} ${styles.drawdownChart}`}>
      <div>
        <span>回撤开始</span>
        <strong>{formatNanos(analysis.risk.max_drawdown_started_at)}</strong>
      </div>
      <svg
        viewBox="0 0 600 120"
        preserveAspectRatio="none"
        role="img"
        aria-label="账户权益回撤随回测时间变化"
      >
        <line x1="10" y1="15" x2="590" y2="15" />
        <polyline points={points} />
      </svg>
      <div>
        <span>回撤低点</span>
        <strong>{formatNanos(analysis.risk.max_drawdown_trough_at)}</strong>
      </div>
    </div>
  );
}

function SourceHash({
  label,
  reference,
  hash,
}: {
  label: string;
  reference: string;
  hash: string;
}) {
  return (
    <div>
      <span>{label}</span>
      <strong title={hash}>{reference}</strong>
    </div>
  );
}

function formatRate(value: string): string {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? `${(parsed * 100).toFixed(2)}%` : "不可计算";
}

function eventTypeLabel(value: string): string {
  return (
    {
      run_started: "运行开始",
      equity_updated: "权益更新",
      trade_filled: "成交",
      position_opened: "开仓",
      position_closed: "平仓",
      run_completed: "运行完成",
    }[value] ?? value
  );
}

function BacktestReport({
  mode = "Backtest",
  trades,
  positions,
  equity,
}: {
  mode?: string;
  trades: ReportTrade[];
  positions: ReportPosition[];
  equity: ReportEquityPoint[];
}) {
  return (
    <div className={styles.reportGrid} aria-label={`${mode} 结果明细`}>
      <section
        className={`${styles.panel} ${styles.reportWide}`}
        aria-label={`${mode} 资金曲线`}
      >
        <header>
          <div>
            <span className="eyebrow">{mode} 账户权益</span>
            <h2>资金曲线</h2>
          </div>
          <span>{equity[0]?.currency ?? "--"}</span>
        </header>
        <EquityCurve points={equity} />
      </section>

      <section className={styles.panel} aria-label={`${mode} 成交明细`}>
        <header>
          <div>
            <span className="eyebrow">{mode} 成交记录</span>
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

      <section className={styles.panel} aria-label={`${mode} 持仓明细`}>
        <header>
          <div>
            <span className="eyebrow">{mode} 持仓周期</span>
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

type ReportTrade = {
  trade_id: string;
  ts_event: string;
  side: string;
  quantity: string;
  price: string;
  commission: string | null;
};

type ReportPosition = {
  position_id: string;
  entry_side: string;
  avg_price_open: string;
  avg_price_close: string | null;
  realized_pnl: string | null;
};

type ReportEquityPoint = BacktestEquityPoint;

function EquityCurve({ points }: { points: ReportEquityPoint[] }) {
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
