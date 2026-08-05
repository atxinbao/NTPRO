import { statusLabel } from "../api/mvpStatus";
import { useMvpStatus } from "../features/status/useMvpStatus";
import styles from "./Pages.module.css";

export function OverviewPage() {
  const query = useMvpStatus();
  const data = query.error ? undefined : query.data;

  return (
    <>
      <header className={styles.pageHeading}>
        <div>
          <span className="eyebrow">策略总览</span>
          <h1>
            {data
              ? `${data.strategyId} / ${data.strategyInstanceId}`
              : "等待策略运行状态"}
          </h1>
          <p>同一不可变策略版本的 Backtest、Demo 与 Live 运行上下文。</p>
        </div>
        <span className={styles.blockedBadge}>交易准备度阻断</span>
      </header>

      <section
        className={`${styles.connectionBanner} ${data ? styles.connectionReady : styles.connectionBlocked}`}
        aria-live="polite"
      >
        <div>
          <strong>
            {query.isPending
              ? "正在读取共享状态"
              : data
                ? "策略状态已验证"
                : "策略工作台已阻断"}
          </strong>
          <span>
            {data
              ? `${data.identityContractId} · ${data.business.sourceRef}`
              : query.error instanceof Error
                ? query.error.message
                : "等待只读合同验证"}
          </span>
        </div>
        <em>{query.isPending ? "读取中" : data ? "只读" : "阻断"}</em>
      </section>

      <section className={styles.metricGrid} aria-label="当前运行摘要">
        <Metric
          label="当前 Run"
          value={data?.strategyInstanceId ?? "未加载"}
          note={data ? statusLabel(data.axes.runtime.status) : "状态未知"}
        />
        <Metric
          label="研究引用"
          value={data ? statusLabel(data.axes.research.status) : "未验证"}
          note={data?.backtestRunId ?? "Backtest 引用未知"}
        />
        <Metric
          label="技术健康"
          value={data ? statusLabel(data.axes.technicalHealth.status) : "未知"}
          note={
            data ? statusLabel(data.axes.technicalHealth.freshness) : "时效未知"
          }
        />
        <Metric
          label="Live 准入"
          value="未开放"
          note="真实交易权限为 false"
          warning
        />
      </section>

      <div className={styles.mainGrid}>
        <section className={styles.panel}>
          <header>
            <div>
              <span className="eyebrow">三模式闭环</span>
              <h2>当前版本运行状态</h2>
            </div>
            <span>只读桥接</span>
          </header>
          <div className={styles.modeProgress}>
            <ModeStep
              number="01"
              label="Backtest"
              detail={data?.backtestRunId ?? "等待历史引用"}
              state="历史"
            />
            <ModeStep
              number="02"
              label="Demo"
              detail={
                data
                  ? statusLabel(data.axes.runtime.status)
                  : "等待 Sandbox 状态"
              }
              state="当前"
              active
            />
            <ModeStep
              number="03"
              label="Live"
              detail="真实适配器、账户与权限未开放"
              state="阻断"
              blocked
            />
          </div>
          <div className={styles.tableWrap}>
            <table>
              <thead>
                <tr>
                  <th>运行</th>
                  <th>模式</th>
                  <th>状态</th>
                  <th>账户</th>
                  <th>Venue</th>
                  <th>来源</th>
                </tr>
              </thead>
              <tbody>
                {data ? (
                  <tr>
                    <td>{data.strategyInstanceId}</td>
                    <td>Demo</td>
                    <td>{statusLabel(data.axes.runtime.status)}</td>
                    <td>{data.accountId}</td>
                    <td>{data.venueId}</td>
                    <td>{data.business.sourceRef}</td>
                  </tr>
                ) : (
                  <tr>
                    <td colSpan={6} className="empty">
                      共享状态不可用
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </section>

        <section className={styles.panel}>
          <header>
            <div>
              <span className="eyebrow">当前判断</span>
              <h2>状态与边界</h2>
            </div>
          </header>
          <div className={styles.axisList}>
            <Axis label="研究状态" value={data?.axes.research.status} />
            <Axis label="运行状态" value={data?.axes.runtime.status} />
            <Axis label="技术健康" value={data?.axes.technicalHealth.status} />
            <Axis
              label="交易准备度"
              value={data?.axes.tradingReadiness.status}
              blocked
            />
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

function Axis({
  label,
  value,
  blocked,
}: {
  label: string;
  value?: string;
  blocked?: boolean;
}) {
  return (
    <div className={styles.axisItem}>
      <div>
        <strong>{label}</strong>
        <small>{value ? statusLabel(value) : "共享状态不可用"}</small>
      </div>
      <span className={blocked ? styles.warning : ""}>
        {blocked ? "阻断" : value ? "已验证" : "未知"}
      </span>
    </div>
  );
}
