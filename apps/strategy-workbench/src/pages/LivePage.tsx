import { Activity, KeyRound, ShieldAlert, Unplug } from "lucide-react";

import {
  useLiveAdmission,
  useOverviewProductContext,
} from "../features/product/useProductResources";
import { ProductErrorState, ProductLoading } from "./ProductState";
import styles from "./Pages.module.css";

const blockerLabels: Record<string, string> = {
  independent_owner_approval_missing: "尚未获得 Live 独立审批",
  production_network_not_authorized: "生产网络连接尚未授权",
  authenticated_account_read_not_authorized: "账户只读连接尚未授权",
  live_run_creation_not_authorized: "Live Run 创建尚未授权",
  order_lifecycle_not_authorized: "真实订单生命周期尚未授权",
  automatic_recovery_not_authorized: "自动恢复尚未授权",
  api_key_missing: "生产 API Key 尚未配置",
  api_secret_missing: "生产 API Secret 尚未配置",
};

export function LivePage() {
  const product = useOverviewProductContext();
  const strategyId = product.isReady
    ? product.strategy?.strategy_id
    : undefined;
  const versionId = product.isReady
    ? product.version?.strategy_version_id
    : undefined;
  const admission = useLiveAdmission(strategyId, versionId);
  const error = product.error ?? admission.error;

  if (error) return <ProductErrorState error={error} />;
  if (product.isVerifying || !product.isReady || admission.isPending) {
    return <ProductLoading label="正在验证 Live 独立准入" />;
  }
  if (!admission.data) {
    return <ProductErrorState error={new Error("Live 准入状态不可用")} />;
  }

  const data = admission.data.data;
  const boundaries = admission.data.boundaries;

  return (
    <>
      <header className={styles.pageHeading}>
        <div>
          <span className="eyebrow">Live</span>
          <h1>真实交易独立准入</h1>
          <p>先核对生产 Venue、账户、凭证和订单边界，再进入后续连接验收。</p>
        </div>
        <span className={styles.readOnlyBadge}>
          <ShieldAlert aria-hidden="true" /> 当前阻断
        </span>
      </header>

      <section className={styles.metricGrid} aria-label="Live 准入摘要">
        <article>
          <span>准入状态</span>
          <strong>
            {data.admission_status === "blocked" ? "未准入" : "未知"}
          </strong>
          <small>不继承 Backtest 或 Demo 权限</small>
        </article>
        <article>
          <span>生产 Venue</span>
          <strong>{data.venue.venue_id}</strong>
          <small>{data.venue.product_type.toUpperCase()}</small>
        </article>
        <article>
          <span>连接状态</span>
          <strong>未尝试</strong>
          <small>本任务未发起外部网络请求</small>
        </article>
        <article>
          <span>真实订单</span>
          <strong>关闭</strong>
          <small>提交、撤单、改单均阻断</small>
        </article>
      </section>

      <section className={styles.backtestLayout}>
        <div className={styles.panel}>
          <header>
            <div>
              <span className="eyebrow">生产目标</span>
              <h2>{data.strategy_version_id}</h2>
            </div>
            <Unplug aria-hidden="true" />
          </header>
          <div className={styles.versionSummary}>
            <Detail label="账户引用" value={data.account.account_ref} />
            <Detail label="账户状态" value="已登记，未授权" />
            <Detail
              label="市场数据适配器"
              value={data.venue.market_data_adapter_ref}
            />
            <Detail
              label="执行适配器"
              value={data.venue.execution_adapter_ref}
            />
            <Detail
              label="HTTP Endpoint"
              value={data.venue.production_http_base_url}
            />
            <Detail
              label="WebSocket Endpoint"
              value={data.venue.production_websocket_base_url}
            />
          </div>
        </div>

        <aside className={styles.backtestBoundary} aria-label="凭证与能力边界">
          <span className="eyebrow">凭证与门禁</span>
          <h2>
            <KeyRound aria-hidden="true" /> 仅检查是否配置
          </h2>
          <Boundary
            label="API Key"
            value={presenceLabel(data.credentials.api_key_presence)}
          />
          <Boundary
            label="API Secret"
            value={presenceLabel(data.credentials.api_secret_presence)}
          />
          <Boundary label="账户只读" value="关闭" />
          <Boundary label="Live Run 创建" value="关闭" />
          <Boundary label="订单提交" value="关闭" />
          <Boundary label="撤单与改单" value="关闭" />
          <Boundary label="自动恢复" value="关闭" />
          <Boundary label="人工停机" value="必须" enabled />
        </aside>
      </section>

      <section className={styles.panel} aria-label="当前阻断原因">
        <header>
          <div>
            <span className="eyebrow">准入清单</span>
            <h2>后续任务必须逐项解除</h2>
          </div>
          <Activity aria-hidden="true" />
        </header>
        <div className={styles.detailGrid}>
          {data.blockers.map((blocker) => (
            <div className={styles.detailRow} key={blocker}>
              <span>{blockerLabels[blocker] ?? blocker}</span>
              <strong>阻断</strong>
            </div>
          ))}
        </div>
        <p>
          当前页面只展示准入事实。外部连接：
          {boundaries.external_venue_connection ? "已开启" : "关闭"}；真实订单：
          {boundaries.real_orders_submitted ? "已发生" : "未发生"}。
        </p>
      </section>
    </>
  );
}

function presenceLabel(value: "missing" | "present"): string {
  return value === "present" ? "已配置" : "缺失";
}

function Detail({ label, value }: { label: string; value: string }) {
  return (
    <div className={styles.detailRow}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function Boundary({
  label,
  value,
  enabled = false,
}: {
  label: string;
  value: string;
  enabled?: boolean;
}) {
  return (
    <div>
      <span>{label}</span>
      <strong className={enabled ? styles.boundaryEnabled : undefined}>
        {value}
      </strong>
    </div>
  );
}
