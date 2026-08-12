import {
  Activity,
  KeyRound,
  RefreshCw,
  ShieldAlert,
  Unplug,
} from "lucide-react";
import { useState } from "react";

import {
  useLiveAdmission,
  useLiveRunCandidates,
  useCreateLiveRunCandidate,
  useLiveRunCandidateAction,
  useOverviewProductContext,
  useRefreshLiveAccount,
} from "../features/product/useProductResources";
import { ProductErrorState, ProductLoading } from "./ProductState";
import styles from "./Pages.module.css";

const blockerLabels: Record<string, string> = {
  independent_owner_approval_missing: "尚未获得 Live 独立审批",
  production_network_not_authorized: "生产网络连接尚未授权",
  authenticated_account_read_not_authorized: "账户只读连接尚未授权",
  live_run_creation_not_authorized: "真实 Live Runtime 启动尚未授权",
  order_lifecycle_not_authorized: "真实订单生命周期尚未授权",
  automatic_recovery_not_authorized: "自动恢复尚未授权",
  api_key_missing: "生产 API Key 尚未配置",
  api_secret_missing: "生产 API Secret 尚未配置",
};

export function LivePage() {
  const [liveRunConfirmed, setLiveRunConfirmed] = useState(false);
  const product = useOverviewProductContext();
  const strategyId = product.isReady
    ? product.strategy?.strategy_id
    : undefined;
  const versionId = product.isReady
    ? product.version?.strategy_version_id
    : undefined;
  const admission = useLiveAdmission(strategyId, versionId);
  const liveRunCandidates = useLiveRunCandidates();
  const accountRefresh = useRefreshLiveAccount();
  const createLiveRun = useCreateLiveRunCandidate();
  const liveRunAction = useLiveRunCandidateAction();
  const error = product.error ?? admission.error ?? liveRunCandidates.error;

  if (error) return <ProductErrorState error={error} />;
  if (
    product.isVerifying ||
    !product.isReady ||
    admission.isPending ||
    liveRunCandidates.isPending
  ) {
    return <ProductLoading label="正在验证 Live 独立准入" />;
  }
  if (!admission.data) {
    return <ProductErrorState error={new Error("Live 准入状态不可用")} />;
  }

  const data = admission.data.data;
  const boundaries = admission.data.boundaries;
  const account = accountRefresh.data?.data;
  const liveRun = liveRunCandidates.data?.data[0];
  const canCreateLiveRun =
    account?.connection_status === "connected" &&
    account.account_result?.can_trade === true;
  const connectionLabel = account
    ? account.connection_status === "connected"
      ? "已连接"
      : account.connection_status === "failed"
        ? "连接失败"
        : "已阻断"
    : "未检查";

  return (
    <>
      <header className={styles.pageHeading}>
        <div>
          <span className="eyebrow">Live</span>
          <h1>Live 连接与独立准入</h1>
          <p>生产账户只读连接独立授权，真实订单生命周期继续关闭。</p>
        </div>
        <span className={styles.readOnlyBadge}>
          <ShieldAlert aria-hidden="true" /> 只读边界
        </span>
      </header>

      <section className={styles.metricGrid} aria-label="Live 准入摘要">
        <article>
          <span>准入状态</span>
          <strong>
            {data.admission_status === "read_only_ready"
              ? "只读就绪"
              : "未准入"}
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
          <strong>{connectionLabel}</strong>
          <small>
            {account?.latency_ms === null || account?.latency_ms === undefined
              ? "等待显式检查"
              : `${account.latency_ms} ms`}
          </small>
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
            <Detail
              label="账户状态"
              value={
                data.account.authenticated_read_state === "ready"
                  ? "只读授权就绪"
                  : "已登记，未授权"
              }
            />
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
          <Boundary
            label="账户只读"
            value={
              boundaries.authenticated_account_read_allowed ? "就绪" : "关闭"
            }
            enabled={boundaries.authenticated_account_read_allowed}
          />
          <Boundary label="真实 Runtime 启动" value="关闭" />
          <Boundary label="订单提交" value="关闭" />
          <Boundary label="撤单与改单" value="关闭" />
          <Boundary label="自动恢复" value="关闭" />
          <Boundary label="人工停机" value="必须" enabled />
        </aside>
      </section>

      <section className={styles.panel} aria-label="生产账户只读连接">
        <header>
          <div>
            <span className="eyebrow">账户连接</span>
            <h2>Binance Spot 生产只读检查</h2>
          </div>
          <button
            className={styles.liveRefreshButton}
            type="button"
            disabled={accountRefresh.isPending || !strategyId || !versionId}
            onClick={() =>
              accountRefresh.mutate({
                strategyId: strategyId!,
                versionId: versionId!,
              })
            }
          >
            <RefreshCw aria-hidden="true" />
            {accountRefresh.isPending ? "检查中" : "检查账户连接"}
          </button>
        </header>
        {accountRefresh.error ? (
          <p className={styles.formError}>{accountRefresh.error.message}</p>
        ) : null}
        <div className={styles.detailGrid} aria-live="polite">
          <Detail label="连接结果" value={connectionLabel} />
          <Detail
            label="运行授权"
            value={
              account
                ? `${5 - account.missing_runtime_gate_refs.length}/5`
                : "未检查"
            }
          />
          <Detail
            label="网络请求"
            value={account?.network_attempted ? "已尝试" : "未尝试"}
          />
          <Detail
            label="响应验证"
            value={account?.response_shape_validated ? "通过" : "未通过"}
          />
          <Detail
            label="非零资产"
            value={
              account?.funds_summary.non_zero_asset_count.toString() ?? "未返回"
            }
          />
          <Detail
            label="已省略零余额"
            value={
              account?.funds_summary.zero_balance_entry_count?.toString() ??
              "未返回"
            }
          />
        </div>
        {account?.account_result ? (
          <>
            <div className={styles.detailGrid} aria-label="Live 账户摘要">
              <Detail
                label="账户类型"
                value={account.account_result.account_type}
              />
              <Detail
                label="交易所交易权限"
                value={permissionLabel(account.account_result.can_trade)}
              />
              <Detail
                label="交易所充值权限"
                value={permissionLabel(account.account_result.can_deposit)}
              />
              <Detail
                label="交易所提现权限"
                value={permissionLabel(account.account_result.can_withdraw)}
              />
            </div>
            <div className={styles.tableWrap} data-testid="live-asset-balances">
              <table className={styles.detailTable}>
                <thead>
                  <tr>
                    <th>资产</th>
                    <th>可用</th>
                    <th>锁定</th>
                    <th>总额</th>
                  </tr>
                </thead>
                <tbody>
                  {account.asset_balances.map((balance) => (
                    <tr key={balance.asset}>
                      <td>{balance.asset}</td>
                      <td>{balance.free}</td>
                      <td>{balance.locked}</td>
                      <td>{balance.total}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </>
        ) : null}
        <p>
          账户读取：{account?.account_read_attempted ? "已执行" : "未执行"}；
          {account?.account_result
            ? "余额使用各资产原生单位，未做跨币种估值；"
            : "账户结果尚未返回；"}
          原始账户响应：不暴露；NTPRO 订单接口：关闭；自动重试：关闭。
        </p>
      </section>

      <section className={styles.panel} aria-label="Live Run 候选">
        <header>
          <div>
            <span className="eyebrow">Live Run</span>
            <h2>启动前检查与人工停止</h2>
          </div>
          <span className={styles.readOnlyBadge}>订单发送关闭</span>
        </header>
        {liveRun ? (
          <>
            <div className={styles.detailGrid}>
              <Detail label="Run ID" value={liveRun.run_id} />
              <Detail label="生命周期" value={liveRun.lifecycle} />
              <Detail
                label="账户连接"
                value={liveRun.account_connected ? "已验证" : "待检查"}
              />
              <Detail
                label="交易权限"
                value={liveRun.account_can_trade_verified ? "已验证" : "待检查"}
              />
              <Detail label="真实 Runtime" value="未启动" />
              <Detail label="订单准入" value="已阻断" />
              <Detail
                label="外部审计锚点"
                value={
                  liveRun.audit_anchor.status ===
                  "verified_external_monotonic_anchor"
                    ? "已验证"
                    : "已阻断"
                }
              />
              <Detail
                label="审计 Revision"
                value={liveRun.audit_anchor.revision.toString()}
              />
              <Detail
                label="回执引用"
                value={liveRun.audit_anchor.receipt_ref}
              />
              <Detail label="审计 Key" value={liveRun.audit_anchor.key_id} />
            </div>
            {liveRunAction.error ? (
              <p className={styles.formError}>{liveRunAction.error.message}</p>
            ) : null}
            <div className={styles.runActions}>
              <span>检查通过只代表候选就绪，不会连接行情或发送订单。</span>
              {liveRun.lifecycle === "created" ? (
                <button
                  type="button"
                  disabled={liveRunAction.isPending}
                  onClick={() =>
                    liveRunAction.mutate({
                      runId: liveRun.run_id,
                      action: "preflight",
                    })
                  }
                >
                  执行启动前检查
                </button>
              ) : null}
              {liveRun.lifecycle !== "stopped" ? (
                <button
                  type="button"
                  disabled={liveRunAction.isPending}
                  onClick={() =>
                    liveRunAction.mutate({
                      runId: liveRun.run_id,
                      action: "stop",
                    })
                  }
                >
                  人工停止候选
                </button>
              ) : null}
            </div>
          </>
        ) : (
          <>
            <p>
              先显式检查生产账户。账户连接与交易所交易权限都验证后，才可创建候选。
            </p>
            <label className={styles.confirmationRow}>
              <input
                type="checkbox"
                checked={liveRunConfirmed}
                onChange={(event) => setLiveRunConfirmed(event.target.checked)}
              />
              <span>
                我确认创建 Live Run 候选；当前不会启动 runtime 或发送订单。
              </span>
            </label>
            {createLiveRun.error ? (
              <p className={styles.formError}>{createLiveRun.error.message}</p>
            ) : null}
            <div className={styles.runActions}>
              <span>独立门禁由服务端再次校验，前端状态不能授权。</span>
              <button
                type="button"
                disabled={
                  !canCreateLiveRun ||
                  !liveRunConfirmed ||
                  createLiveRun.isPending
                }
                onClick={() =>
                  createLiveRun.mutate({
                    strategy_id: strategyId!,
                    strategy_version_id: versionId!,
                    environment: "live",
                    account_ref: "account://live/binance/primary",
                    venue_ref: "venue://live/BINANCE",
                    user_confirmed: true,
                  })
                }
              >
                创建 Live Run 候选
              </button>
            </div>
          </>
        )}
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

function permissionLabel(enabled: boolean): string {
  return enabled ? "交易所允许" : "交易所关闭";
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
