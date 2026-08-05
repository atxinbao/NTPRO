import { statusLabel } from "../api/mvpStatus";
import { useMvpStatus } from "../features/status/useMvpStatus";
import styles from "./Pages.module.css";

export function SystemStatusPage() {
  const query = useMvpStatus();
  const data = query.error ? undefined : query.data;
  const axes = data
    ? ([
        ["研究状态", data.axes.research],
        ["运行状态", data.axes.runtime],
        ["技术健康", data.axes.technicalHealth],
        ["交易准备度", data.axes.tradingReadiness],
      ] as const)
    : [];

  return (
    <>
      <header className={styles.pageHeading}>
        <div>
          <span className="eyebrow">辅助诊断</span>
          <h1>系统状态</h1>
          <p>当前 StrategyVersion 与 Sandbox 运行实例的只读状态。</p>
        </div>
      </header>
      <div className={styles.statusGrid}>
        <section className={styles.panel}>
          <header>
            <div>
              <span className="eyebrow">四轴合同</span>
              <h2>运行判断</h2>
            </div>
            <span>{data ? "已验证" : "不可用"}</span>
          </header>
          <div className={styles.axisList}>
            {axes.map(([label, value]) => (
              <div className={styles.axisItem} key={label}>
                <div>
                  <strong>{label}</strong>
                  <small>
                    {statusLabel(value.availability)} ·{" "}
                    {statusLabel(value.freshness)}
                  </small>
                </div>
                <span
                  className={value.status === "blocked" ? styles.warning : ""}
                >
                  {statusLabel(value.status)}
                </span>
              </div>
            ))}
            {!data ? (
              <p className="empty">
                {query.isPending
                  ? "正在读取共享状态"
                  : "合同校验失败，旧状态已清空"}
              </p>
            ) : null}
          </div>
        </section>
        <section className={styles.panel}>
          <header>
            <div>
              <span className="eyebrow">来源</span>
              <h2>状态证据</h2>
            </div>
          </header>
          <div className={styles.sourceRows}>
            <SourceRow
              label="身份合同"
              value={data?.identityContractId ?? "未知"}
            />
            <SourceRow label="节点" value={data?.nodeId ?? "未知"} />
            <SourceRow
              label="业务快照"
              value={data?.business.sourceRef ?? "未知"}
            />
            <SourceRow
              label="更新时间"
              value={
                data
                  ? new Date(data.generatedAtUnixMs).toLocaleString("zh-CN")
                  : "未知"
              }
            />
            {(data?.sourceRefs ?? []).map((source, index) => (
              <SourceRow
                key={source}
                label={`来源 ${index + 1}`}
                value={source}
              />
            ))}
          </div>
        </section>
      </div>
    </>
  );
}

function SourceRow({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
