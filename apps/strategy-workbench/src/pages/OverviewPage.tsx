import { Link } from "@tanstack/react-router";
import { ArrowUpRight, LockKeyhole } from "lucide-react";

import type { Run, RunEnvironment } from "../api/generated/productApi";
import {
  environmentLabels,
  formatTimestamp,
  lifecycleLabels,
  productErrorMessage,
  resultLabels,
  riskLabels,
} from "../features/product/presentation";
import { useOverviewProductContext } from "../features/product/useProductResources";
import { ProductErrorState, ProductLoading } from "./ProductState";
import styles from "./Pages.module.css";

const environments: RunEnvironment[] = ["backtest", "sandbox", "live"];

export function OverviewPage() {
  const product = useOverviewProductContext();

  if (product.error) return <ProductErrorState error={product.error} />;
  if (product.isVerifying || !product.isReady) {
    return <ProductLoading label="正在验证策略产品资源" />;
  }
  if (!product.strategy || !product.strategies) {
    return <EmptyOverview />;
  }

  const currentVersion = product.version;
  const runItems = product.runs?.data ?? [];
  const runtimeMessage = product.runtimeError
    ? productErrorMessage(product.runtimeError)
    : undefined;

  return (
    <>
      <header className={styles.pageHeading}>
        <div>
          <span className="eyebrow">策略总览</span>
          <h1>{product.strategy.name}</h1>
          <p>{product.strategy.description}</p>
        </div>
        <span className={styles.readOnlyBadge}>
          <LockKeyhole aria-hidden="true" /> 只读产品资源
        </span>
      </header>

      <section
        className={`${styles.connectionBanner} ${runtimeMessage ? styles.connectionBlocked : styles.connectionReady}`}
        aria-live="polite"
      >
        <div>
          <strong>
            {runtimeMessage ? "策略目录已验证，运行数据降级" : "产品资源已验证"}
          </strong>
          <span>
            {runtimeMessage
              ? runtimeMessage.detail
              : `${product.strategies.contract_version} · 请求 ${product.strategies.request_id}`}
          </span>
        </div>
        <em>{runtimeMessage ? "运行源不可用" : "来源新鲜"}</em>
      </section>

      <section className={styles.metricGrid} aria-label="策略摘要">
        <Metric
          label="策略 ID"
          value={product.strategy.strategy_id}
          note={product.strategy.owner}
        />
        <Metric
          label="当前版本"
          value={currentVersion?.version ?? "未注册"}
          note={currentVersion ? "不可变版本精确读取" : "状态未知"}
        />
        <Metric
          label="当前页 Run"
          value={
            runtimeMessage
              ? "--"
              : `${product.runs?.page.returned_count ?? 0}${product.runs?.page.has_more ? "+" : ""}`
          }
          note={
            runtimeMessage
              ? "运行数据源暂不可用"
              : product.runs?.page.has_more
                ? "还有下一页"
                : "当前版本全部 Run"
          }
          warning={Boolean(runtimeMessage)}
        />
        <Metric
          label="交易能力"
          value="全部关闭"
          note="只读合同已验证"
          warning
        />
      </section>

      <div className={styles.mainGrid}>
        <section className={styles.panel}>
          <header>
            <div>
              <span className="eyebrow">三模式运行</span>
              <h2>当前版本 Run</h2>
            </div>
            <span>{currentVersion?.strategy_version_id}</span>
          </header>
          <div className={styles.modeProgress}>
            {environments.map((environment, index) => {
              const modeRuns = runItems.filter(
                (run) => run.environment === environment,
              );
              const latest = modeRuns[0];
              return (
                <ModeStep
                  key={environment}
                  number={String(index + 1).padStart(2, "0")}
                  label={environmentLabels[environment]}
                  detail={
                    latest ? lifecycleLabels[latest.lifecycle] : "暂无 Run"
                  }
                  state={String(modeRuns.length)}
                  active={latest?.lifecycle === "running"}
                  blocked={Boolean(
                    latest &&
                    (latest.risk.status === "blocked" ||
                      ["failed", "cancelled", "stopped"].includes(
                        latest.lifecycle,
                      )),
                  )}
                />
              );
            })}
          </div>
          <RunTable runs={runItems} runtimeError={runtimeMessage?.title} />
        </section>

        <section className={styles.panel}>
          <header>
            <div>
              <span className="eyebrow">不可变版本</span>
              <h2>{currentVersion?.strategy_version_id ?? "版本未注册"}</h2>
            </div>
            <span>
              {product.versions?.page.returned_count ?? 0}
              {product.versions?.page.has_more ? "+" : ""} 个当前页版本
            </span>
          </header>
          {currentVersion ? (
            <div className={styles.versionSummary}>
              <KeyValue
                label="内容 Hash"
                value={currentVersion.content_hash}
                mono
              />
              <KeyValue label="代码引用" value={currentVersion.code_ref} mono />
              <KeyValue
                label="交易标的"
                value={currentVersion.data_requirements.symbols.join("、")}
              />
              <KeyValue
                label="数据类型"
                value={currentVersion.data_requirements.data_types.join("、")}
              />
              <KeyValue
                label="确定性回放"
                value={
                  currentVersion.data_requirements.deterministic_replay_required
                    ? "必须"
                    : "未要求"
                }
              />
              <KeyValue
                label="Kill Switch"
                value={
                  currentVersion.risk_config.kill_switch_required
                    ? "必须"
                    : "未要求"
                }
              />
            </div>
          ) : (
            <div className="empty">默认版本精确查询未完成</div>
          )}
        </section>
      </div>
    </>
  );
}

function EmptyOverview() {
  return (
    <section className={styles.productState} aria-live="polite">
      <div>
        <strong>当前没有已注册策略</strong>
        <span>Product API 返回了经过验证的空列表。</span>
      </div>
    </section>
  );
}

function RunTable({
  runs,
  runtimeError,
}: {
  runs: Run[];
  runtimeError?: string;
}) {
  return (
    <div className={styles.tableWrap} data-testid="run-table-scroll">
      <table className={styles.runTable}>
        <thead>
          <tr>
            <th>Run</th>
            <th>模式</th>
            <th>生命周期</th>
            <th>风险</th>
            <th>结果</th>
            <th>更新时间</th>
          </tr>
        </thead>
        <tbody>
          {runtimeError ? (
            <tr>
              <td colSpan={6} className="empty">
                {runtimeError}，策略与版本目录仍可查看
              </td>
            </tr>
          ) : runs.length > 0 ? (
            runs.map((run) => (
              <tr key={run.run_id}>
                <td>
                  <Link
                    to="/runs/$runId"
                    params={{ runId: run.run_id }}
                    className={styles.runLink}
                  >
                    {run.run_id}
                    <ArrowUpRight aria-hidden="true" />
                  </Link>
                </td>
                <td>{environmentLabels[run.environment]}</td>
                <td>{lifecycleLabels[run.lifecycle]}</td>
                <td
                  className={
                    run.risk.status === "blocked" ? styles.warning : ""
                  }
                >
                  {riskLabels[run.risk.status]}
                </td>
                <td>{resultLabels[run.result.status]}</td>
                <td>{formatTimestamp(run.updated_at_unix_ms)}</td>
              </tr>
            ))
          ) : (
            <tr>
              <td colSpan={6} className="empty">
                当前版本还没有 Run
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
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
      <strong>{value}</strong>
      <small className={warning ? styles.warning : ""}>{note}</small>
    </article>
  );
}

function ModeStep({
  number,
  label,
  detail,
  state,
  active,
  blocked,
}: {
  number: string;
  label: string;
  detail: string;
  state: string;
  active?: boolean;
  blocked?: boolean;
}) {
  return (
    <article
      className={`${active ? styles.modeStepActive : ""} ${blocked ? styles.modeStepBlocked : ""}`}
    >
      <span>{number}</span>
      <div>
        <strong>{label}</strong>
        <small>{detail}</small>
      </div>
      <em>{state}</em>
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
