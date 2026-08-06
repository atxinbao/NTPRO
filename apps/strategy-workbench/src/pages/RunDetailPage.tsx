import { Link, useParams } from "@tanstack/react-router";
import { ArrowLeft, LockKeyhole } from "lucide-react";

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
