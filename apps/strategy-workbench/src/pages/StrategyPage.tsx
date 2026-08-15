import { Link } from "@tanstack/react-router";
import { ArrowUpRight, LockKeyhole } from "lucide-react";

import type { RunEnvironment } from "../api/generated/productApi";
import {
  environmentLabels,
  formatTimestamp,
  lifecycleLabels,
  productErrorMessage,
} from "../features/product/presentation";
import { useOverviewProductContext } from "../features/product/useProductResources";
import { ProductErrorState, ProductLoading } from "./ProductState";
import styles from "./Pages.module.css";

const environments: RunEnvironment[] = ["backtest", "sandbox", "live"];

const environmentRoutes: Record<
  RunEnvironment,
  "/backtests" | "/demo" | "/live"
> = {
  backtest: "/backtests",
  sandbox: "/demo",
  live: "/live",
};

const strategyLifecycleLabels = {
  draft: "草稿",
  active: "已启用",
  archived: "已归档",
} as const;

export function StrategyPage() {
  const product = useOverviewProductContext();

  if (product.error) return <ProductErrorState error={product.error} />;
  if (product.isVerifying || !product.isReady) {
    return <ProductLoading label="正在验证策略目录" />;
  }
  if (!product.strategy || !product.strategies) {
    return (
      <section className={styles.productState} aria-live="polite">
        <div>
          <strong>当前没有已注册策略</strong>
          <span>Product API 返回了经过验证的空策略目录。</span>
        </div>
      </section>
    );
  }

  const strategy = product.strategy;
  const version = product.version;
  const runs = product.runs?.data ?? [];
  const runtimeMessage = product.runtimeError
    ? productErrorMessage(product.runtimeError)
    : undefined;
  const parameters = version ? fixedParameters(version.parameter_schema) : [];

  return (
    <>
      <header className={styles.pageHeading}>
        <div>
          <span className="eyebrow">策略目录</span>
          <h1>策略管理</h1>
          <p>
            查看策略身份、不可变版本、数据要求和 Backtest、Demo、Live 运行状态。
          </p>
        </div>
        <span className={styles.readOnlyBadge}>
          <LockKeyhole aria-hidden="true" /> 只读策略资源
        </span>
      </header>

      <section
        className={`${styles.connectionBanner} ${runtimeMessage ? styles.connectionBlocked : styles.connectionReady}`}
        aria-live="polite"
      >
        <div>
          <strong>
            {runtimeMessage ? "策略目录已验证，运行摘要降级" : "策略资源已验证"}
          </strong>
          <span>
            {runtimeMessage
              ? runtimeMessage.detail
              : `${product.strategies.contract_version} · 请求 ${product.strategies.request_id}`}
          </span>
        </div>
        <em>{runtimeMessage ? "运行源不可用" : "来源新鲜"}</em>
      </section>

      <section className={styles.metricGrid} aria-label="策略目录摘要">
        <Metric
          label="策略数量"
          value={String(product.strategies.page.returned_count)}
          note={
            product.strategies.page.has_more ? "还有下一页" : "当前目录全部策略"
          }
        />
        <Metric
          label="当前策略"
          value={strategy.strategy_id}
          note={strategy.owner}
        />
        <Metric
          label="默认版本"
          value={version?.version ?? "未注册"}
          note={
            version?.status === "registered" ? "不可变版本已登记" : "版本不可用"
          }
        />
        <Metric
          label="生命周期"
          value={strategyLifecycleLabels[strategy.lifecycle]}
          note={`更新于 ${formatTimestamp(strategy.updated_at_unix_ms)}`}
        />
      </section>

      <div className={styles.mainGrid}>
        <section className={styles.panel} aria-label="当前策略身份">
          <header>
            <div>
              <span className="eyebrow">当前策略</span>
              <h2>{strategy.name}</h2>
            </div>
            <span>{strategyLifecycleLabels[strategy.lifecycle]}</span>
          </header>
          <div className={styles.versionSummary}>
            <Detail label="策略 ID" value={strategy.strategy_id} mono />
            <Detail label="说明" value={strategy.description} />
            <Detail label="所有者" value={strategy.owner} />
            <Detail label="默认版本" value={strategy.default_version_id} mono />
            <Detail label="来源类型" value={strategy.source.source_type} mono />
            <Detail label="来源状态" value={strategy.source.freshness_status} />
          </div>
        </section>

        <section className={styles.panel} aria-label="默认不可变版本">
          <header>
            <div>
              <span className="eyebrow">默认不可变版本</span>
              <h2>{version?.strategy_version_id ?? "版本未注册"}</h2>
            </div>
            <span>{version?.status ?? "未知"}</span>
          </header>
          {version ? (
            <div className={styles.versionSummary}>
              <Detail label="内容 Hash" value={version.content_hash} mono />
              <Detail label="代码引用" value={version.code_ref} mono />
              <Detail
                label="风险配置"
                value={version.risk_config.risk_profile_ref}
                mono
              />
              <Detail
                label="Kill Switch"
                value={
                  version.risk_config.kill_switch_required ? "必须" : "未要求"
                }
              />
              <Detail label="外部 Venue 默认" value="关闭" />
              <Detail label="订单提交默认" value="关闭" />
            </div>
          ) : (
            <div className="empty">默认版本精确查询未完成</div>
          )}
        </section>
      </div>

      <div className={styles.detailGrid}>
        <section className={styles.panel} aria-label="策略固定参数">
          <header>
            <div>
              <span className="eyebrow">固定参数</span>
              <h2>版本参数合同</h2>
            </div>
            <span>{parameters.length} 项</span>
          </header>
          <div className={styles.versionSummary}>
            {parameters.length > 0 ? (
              parameters.map(([name, value]) => (
                <Detail key={name} label={name} value={value} mono />
              ))
            ) : (
              <div className="empty">当前版本没有固定参数</div>
            )}
          </div>
        </section>

        <section className={styles.panel} aria-label="策略数据要求">
          <header>
            <div>
              <span className="eyebrow">数据要求</span>
              <h2>确定性输入合同</h2>
            </div>
            <span>
              {version?.data_requirements.deterministic_replay_required
                ? "必须回放"
                : "未要求"}
            </span>
          </header>
          {version ? (
            <div className={styles.versionSummary}>
              <Detail
                label="Venue"
                value={version.data_requirements.venues.join("、")}
              />
              <Detail
                label="交易标的"
                value={version.data_requirements.symbols.join("、")}
              />
              <Detail
                label="数据类型"
                value={version.data_requirements.data_types.join("、")}
              />
              <Detail
                label="时间粒度"
                value={version.data_requirements.timeframes.join("、")}
              />
            </div>
          ) : (
            <div className="empty">当前版本没有数据要求</div>
          )}
        </section>
      </div>

      <section className={styles.panel} aria-label="策略运行模式摘要">
        <header>
          <div>
            <span className="eyebrow">运行模式</span>
            <h2>同一策略版本的三种运行环境</h2>
          </div>
          <span>
            {runtimeMessage ? "运行源不可用" : `${runs.length} 个 Run`}
          </span>
        </header>
        <div className={styles.modeProgress}>
          {environments.map((environment, index) => {
            const environmentRuns = runs.filter(
              (run) => run.environment === environment,
            );
            const latest = environmentRuns[0];
            return (
              <article key={environment}>
                <span>{String(index + 1).padStart(2, "0")}</span>
                <div>
                  <strong>{environmentLabels[environment]}</strong>
                  <small>
                    {latest ? lifecycleLabels[latest.lifecycle] : "暂无 Run"}
                  </small>
                </div>
                <Link
                  className={styles.runLink}
                  to={environmentRoutes[environment]}
                >
                  {environmentRuns.length} 个
                  <ArrowUpRight aria-hidden="true" />
                </Link>
              </article>
            );
          })}
        </div>
      </section>
    </>
  );
}

function fixedParameters(
  schema: Record<string, unknown>,
): Array<[string, string]> {
  const properties = schema.properties;
  if (
    !properties ||
    typeof properties !== "object" ||
    Array.isArray(properties)
  ) {
    return [];
  }
  return Object.entries(properties)
    .map(([name, definition]) => {
      if (
        !definition ||
        typeof definition !== "object" ||
        Array.isArray(definition)
      ) {
        return [name, "已登记"] as [string, string];
      }
      const fixedValue = (definition as Record<string, unknown>).const;
      return [
        name,
        fixedValue === undefined ? "已登记" : String(fixedValue),
      ] as [string, string];
    })
    .sort(([left], [right]) => left.localeCompare(right));
}

function Metric({
  label,
  value,
  note,
}: {
  label: string;
  value: string;
  note: string;
}) {
  return (
    <article>
      <span>{label}</span>
      <strong>{value}</strong>
      <small>{note}</small>
    </article>
  );
}

function Detail({
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
      <strong className={mono ? styles.mono : undefined}>{value}</strong>
    </div>
  );
}
