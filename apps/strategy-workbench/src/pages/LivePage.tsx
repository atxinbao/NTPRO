import {
  Activity,
  KeyRound,
  RefreshCw,
  ShieldAlert,
  Unplug,
} from "lucide-react";
import { useState } from "react";

import { ProductApiRequestError } from "../api/productApi";
import {
  useLiveAdmission,
  useLiveExecutionCancelOwnerApproval,
  useLiveExecutionOwnerApproval,
  useLiveRunCandidates,
  useCreateLiveRunCandidate,
  useDemoRunSnapshot,
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
  follow_up_order_mutation_not_authorized: "追加与改单尚未授权，撤单需独立门禁",
  automatic_recovery_not_authorized: "自动恢复尚未授权",
  api_key_missing: "生产 API Key 尚未配置",
  api_secret_missing: "生产 API Secret 尚未配置",
};

const liveSizingRejectionLabels: Record<string, string> = {
  "live_sizing_decision.evidence_expired": "账户资金或交易规则证据已过期",
  "live_sizing_decision.instrument_id": "策略标的与交易规则不一致",
  "live_sizing_decision.price_tick": "限价不符合交易所价格步长",
  "live_sizing_decision.quantity_step": "策略数量无法按交易所数量步长规范化",
  "live_sizing_decision.min_quantity": "规范化数量低于交易所最小数量",
  "live_sizing_decision.max_quantity": "规范化数量超过交易所最大数量",
  "live_sizing_decision.min_notional": "订单金额低于交易所最小名义金额",
  "live_sizing_decision.account_balance": "账户可用资产不足",
  "live_sizing_decision.account_budget": "订单金额超过账户单笔预算",
  "live_sizing_decision.request_max_notional": "订单金额超过本次请求上限",
  "live_sizing_decision.risk_policy_max_notional": "订单金额超过全局风险上限",
};

function liveExecutionErrorMessage(error: unknown): string {
  if (error instanceof ProductApiRequestError) {
    return liveSizingRejectionLabels[error.field] ?? error.message;
  }
  return error instanceof Error ? error.message : "Live 执行准入失败";
}

export function LivePage() {
  const [liveRunConfirmed, setLiveRunConfirmed] = useState(false);
  const [cancelConfirmed, setCancelConfirmed] = useState(false);
  const [executionDraft, setExecutionDraft] = useState({
    price: "",
    maxNotional: "",
    userConfirmed: false,
  });
  const product = useOverviewProductContext();
  const strategyId = product.isReady
    ? product.strategy?.strategy_id
    : undefined;
  const versionId = product.isReady
    ? product.version?.strategy_version_id
    : undefined;
  const liveRunCandidates = useLiveRunCandidates();
  const pendingExecutionAdmission =
    liveRunCandidates.data?.data[0]?.lifecycle === "preflight_ready" &&
    liveRunCandidates.data.data[0].order_admission.status === "blocked";
  const sourceDemoRun = product.isReady
    ? product.runs?.data
        .filter(
          (run) => run.environment === "sandbox" && run.lifecycle === "stopped",
        )
        .sort(
          (left, right) => right.created_at_unix_ms - left.created_at_unix_ms,
        )[0]
    : undefined;
  const sourceDemoSnapshot = useDemoRunSnapshot(
    sourceDemoRun?.run_id,
    Boolean(sourceDemoRun && pendingExecutionAdmission),
  );
  const admission = useLiveAdmission(strategyId, versionId);
  const accountRefresh = useRefreshLiveAccount();
  const createLiveRun = useCreateLiveRunCandidate();
  const liveRunAction = useLiveRunCandidateAction();
  const executionOwnerApproval = useLiveExecutionOwnerApproval();
  const executionCancelOwnerApproval = useLiveExecutionCancelOwnerApproval();
  const auditAnchorUnavailable =
    liveRunCandidates.error instanceof ProductApiRequestError &&
    liveRunCandidates.error.field === "live_run_audit_anchor_config";
  const error =
    product.error ??
    (pendingExecutionAdmission ? sourceDemoSnapshot.error : null) ??
    admission.error ??
    (auditAnchorUnavailable ? null : liveRunCandidates.error);

  if (error) return <ProductErrorState error={error} />;
  if (
    product.isVerifying ||
    !product.isReady ||
    (Boolean(sourceDemoRun && pendingExecutionAdmission) &&
      sourceDemoSnapshot.isPending) ||
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
  const liveRunBoundaries = liveRunCandidates.data?.boundaries;
  const strategyIntent =
    sourceDemoSnapshot.data?.data.snapshot_status === "frozen"
      ? sourceDemoSnapshot.data.data.latest_order_intent
      : undefined;
  const executionInstrument = strategyIntent?.symbol;
  const executionSide =
    strategyIntent?.side === "buy"
      ? "BUY"
      : strategyIntent && ["sell", "flatten"].includes(strategyIntent.side)
        ? "SELL"
        : undefined;
  const executionConfirmationsComplete = executionDraft.userConfirmed;
  const canCreateLiveRun =
    !auditAnchorUnavailable &&
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
          <p>生产行情与单笔真实限价单分别准入，每次执行都需要独立确认。</p>
        </div>
        <span className={styles.readOnlyBadge}>
          <ShieldAlert aria-hidden="true" /> 独立准入
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
          <strong>
            {liveRun?.order_admission.status === "authorized_single_shot"
              ? "单笔已授权"
              : liveRun?.order_admission.status === "consumed_single_shot"
                ? "单笔已消费"
                : "未授权"}
          </strong>
          <small>人工撤单需双人确认；改单和自动重试阻断</small>
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
          <Boundary label="行情 Runtime 启动" value="可显式启动" enabled />
          <Boundary
            label="订单提交"
            value={
              liveRun?.order_admission.status === "authorized_single_shot"
                ? "单笔已授权"
                : "未授权"
            }
            enabled={
              liveRun?.order_admission.status === "authorized_single_shot"
            }
          />
          <Boundary
            label="人工撤单"
            value={
              liveRunBoundaries?.cancel_order_allowed ? "双人确认" : "关闭"
            }
            enabled={liveRunBoundaries?.cancel_order_allowed}
          />
          <Boundary label="改单" value="关闭" />
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
            <h2>生产 Runtime、单笔执行与人工停止</h2>
          </div>
          <span className={styles.readOnlyBadge}>
            {liveRun?.order_admission.status === "authorized_single_shot"
              ? "单笔订单待启动"
              : "默认禁止下单"}
          </span>
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
              <Detail
                label="真实 Runtime"
                value={liveRun.runtime_started ? "运行中" : "未运行"}
              />
              <Detail
                label="生产行情"
                value={liveRun.market_data_connected ? "已连接" : "未连接"}
              />
              <Detail
                label="Runtime 进程"
                value={liveRun.runtime_process_state}
              />
              <Detail
                label="Runtime 错误"
                value={liveRun.runtime_error ?? "无"}
              />
              <Detail
                label="订单准入"
                value={
                  liveRun.order_admission.status === "authorized_single_shot"
                    ? "单笔已授权"
                    : liveRun.order_admission.status === "consumed_single_shot"
                      ? "单笔已消费"
                      : "未授权"
                }
              />
              <Detail
                label="负责人审批"
                value={
                  liveRun.order_admission.owner_approved ? "已完成" : "待审批"
                }
              />
              <Detail
                label="风控审批"
                value={
                  liveRun.order_admission.risk_approved ? "已完成" : "待审批"
                }
              />
              <Detail
                label="操作员审批"
                value={
                  liveRun.order_admission.operator_approved
                    ? "已完成"
                    : "待审批"
                }
              />
              <Detail
                label="订单生命周期"
                value={liveRun.execution_order?.status ?? "尚未启动"}
              />
              <Detail
                label="Client Order ID"
                value={liveRun.execution_order?.client_order_id ?? "尚未生成"}
              />
              <Detail
                label="Venue Order ID"
                value={liveRun.execution_order?.venue_order_id ?? "尚未返回"}
              />
              <Detail
                label="原始数量"
                value={liveRun.sizing_decision?.source_quantity ?? "-"}
              />
              <Detail
                label="批准数量"
                value={liveRun.sizing_decision?.approved_quantity ?? "-"}
              />
              <Detail
                label="价格步长"
                value={liveRun.sizing_decision?.price_tick ?? "-"}
              />
              <Detail
                label="数量步长"
                value={liveRun.sizing_decision?.quantity_step ?? "-"}
              />
              <Detail
                label="数量范围"
                value={
                  liveRun.sizing_decision
                    ? `${liveRun.sizing_decision.min_quantity} - ${liveRun.sizing_decision.max_quantity}`
                    : "-"
                }
              />
              <Detail
                label="最小名义金额"
                value={liveRun.sizing_decision?.min_notional ?? "-"}
              />
              <Detail
                label="账户预算比例"
                value={
                  liveRun.sizing_decision
                    ? `${Number(liveRun.sizing_decision.max_account_budget_fraction) * 100}%`
                    : "-"
                }
              />
              <Detail
                label="订单名义金额"
                value={liveRun.sizing_decision?.order_notional ?? "-"}
              />
              <Detail
                label="账户预算"
                value={liveRun.sizing_decision?.account_budget_notional ?? "-"}
              />
              <Detail
                label="Sizing 证据有效至"
                value={
                  liveRun.sizing_decision
                    ? new Date(
                        liveRun.sizing_decision.evidence_expires_at_unix_ms,
                      ).toLocaleString("zh-CN")
                    : "-"
                }
              />
              <Detail
                label="执行数量"
                value={liveRun.execution_order?.original_quantity ?? "-"}
              />
              <Detail
                label="累计成交"
                value={liveRun.execution_order?.filled_quantity ?? "-"}
              />
              <Detail
                label="剩余数量"
                value={liveRun.execution_order?.remaining_quantity ?? "-"}
              />
              <Detail
                label="最近人工控制"
                value={liveRun.execution_control?.status ?? "尚未执行"}
              />
              <Detail
                label="追加订单"
                value={
                  liveRun.execution_order?.new_orders_blocked === true
                    ? "已阻断"
                    : "默认阻断"
                }
              />
              <Detail
                label="订单错误"
                value={liveRun.execution_order?.last_error ?? "无"}
              />
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
                label="Run Revision"
                value={liveRun.audit_anchor.revision.toString()}
              />
              <Detail
                label="Workspace Revision"
                value={liveRun.audit_anchor.workspace_revision.toString()}
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
            {executionOwnerApproval.error ? (
              <p className={styles.formError}>
                {liveExecutionErrorMessage(executionOwnerApproval.error)}
              </p>
            ) : null}
            {executionCancelOwnerApproval.error ? (
              <p className={styles.formError}>
                {executionCancelOwnerApproval.error.message}
              </p>
            ) : null}
            {liveRun.lifecycle === "preflight_ready" &&
            liveRun.order_admission.status === "blocked" ? (
              <form
                className={styles.liveExecutionForm}
                aria-label="单笔真实限价单准入"
                onSubmit={(event) => {
                  event.preventDefault();
                  if (
                    !sourceDemoRun ||
                    !strategyIntent ||
                    !executionInstrument ||
                    !executionSide ||
                    !executionConfirmationsComplete
                  ) {
                    return;
                  }
                  executionOwnerApproval.mutate({
                    runId: liveRun.run_id,
                    request: {
                      run_id: liveRun.run_id,
                      strategy_version_id: liveRun.strategy_version_id,
                      account_ref: "account://live/binance/primary",
                      venue_ref: "venue://live/BINANCE",
                      admission_id: `manual-${Date.now()}`,
                      source_demo_run_id: sourceDemoRun.run_id,
                      strategy_intent_id: strategyIntent.intent_id,
                      instrument_id: executionInstrument,
                      side: executionSide,
                      order_type: "LIMIT",
                      time_in_force: "GTC",
                      price: executionDraft.price,
                      quantity: String(strategyIntent.quantity),
                      max_notional: executionDraft.maxNotional,
                      expires_at_unix_ms: Date.now() + 5 * 60 * 1_000,
                      user_confirmed: true,
                    },
                  });
                }}
              >
                <header>
                  <div>
                    <span className="eyebrow">Execution Admission</span>
                    <h3>单笔真实 LIMIT / GTC 订单</h3>
                  </div>
                  <ShieldAlert aria-hidden="true" />
                </header>
                <div className={styles.formGrid}>
                  <label>
                    <span>交易标的</span>
                    <input
                      value={executionInstrument ?? "无可用标的"}
                      readOnly
                    />
                  </label>
                  <label>
                    <span>方向</span>
                    <input value={executionSide ?? "无策略意图"} readOnly />
                  </label>
                  <label>
                    <span>限价</span>
                    <input
                      inputMode="decimal"
                      required
                      value={executionDraft.price}
                      onChange={(event) =>
                        setExecutionDraft((current) => ({
                          ...current,
                          price: event.target.value,
                        }))
                      }
                    />
                  </label>
                  <label>
                    <span>数量</span>
                    <input
                      value={
                        strategyIntent
                          ? String(strategyIntent.quantity)
                          : "无策略意图"
                      }
                      readOnly
                    />
                  </label>
                  <label>
                    <span>来源 Demo Run</span>
                    <input
                      value={sourceDemoRun?.run_id ?? "无已冻结 Demo Run"}
                      readOnly
                    />
                  </label>
                  <label>
                    <span>策略意图</span>
                    <input
                      value={strategyIntent?.intent_id ?? "无策略意图"}
                      readOnly
                    />
                  </label>
                  <label className={styles.formFieldWide}>
                    <span>最大名义金额</span>
                    <input
                      inputMode="decimal"
                      required
                      value={executionDraft.maxNotional}
                      onChange={(event) =>
                        setExecutionDraft((current) => ({
                          ...current,
                          maxNotional: event.target.value,
                        }))
                      }
                    />
                  </label>
                </div>
                <div className={styles.executionConfirmations}>
                  <label>
                    <input
                      type="checkbox"
                      checked={executionDraft.userConfirmed}
                      onChange={(event) =>
                        setExecutionDraft((current) => ({
                          ...current,
                          userConfirmed: event.target.checked,
                        }))
                      }
                    />
                    <span>我以机构负责人身份确认这是一笔真实订单</span>
                  </label>
                </div>
                <footer>
                  <span>提交负责人审批后，仍需风控与当班操作员分别审批。</span>
                  <button
                    type="submit"
                    disabled={
                      !executionInstrument ||
                      !strategyIntent ||
                      !executionConfirmationsComplete ||
                      executionOwnerApproval.isPending
                    }
                  >
                    {executionOwnerApproval.isPending
                      ? "审批中"
                      : "提交负责人审批"}
                  </button>
                </footer>
              </form>
            ) : null}
            <div className={styles.runActions}>
              <span>
                {liveRun.order_admission.status === "authorized_single_shot"
                  ? "已绑定单笔订单，启动后不能追加或改单；撤单需双人确认。"
                  : liveRun.execution_order
                    ? "可人工对账；撤单需机构申请并由当班操作员再次确认。"
                    : "普通行情启动不注册执行客户端。"}
              </span>
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
              {liveRun.lifecycle === "preflight_ready" ? (
                <button
                  type="button"
                  disabled={
                    liveRunAction.isPending ||
                    liveRun.order_admission.status !== "blocked"
                  }
                  onClick={() =>
                    liveRunAction.mutate({
                      runId: liveRun.run_id,
                      action: "start_market_data",
                    })
                  }
                >
                  启动生产行情
                </button>
              ) : null}
              {liveRun.lifecycle === "preflight_ready" &&
              liveRun.order_admission.status === "authorized_single_shot" ? (
                <button
                  type="button"
                  disabled={liveRunAction.isPending}
                  onClick={() =>
                    liveRunAction.mutate({
                      runId: liveRun.run_id,
                      action: "start_execution",
                    })
                  }
                >
                  启动单笔真实执行
                </button>
              ) : null}
              {liveRun.lifecycle === "market_data_running" &&
              liveRun.execution_order?.client_order_id &&
              !liveRun.execution_order.terminal &&
              liveRunBoundaries?.fill_reconciliation_allowed ? (
                <button
                  type="button"
                  disabled={liveRunAction.isPending}
                  onClick={() =>
                    liveRunAction.mutate({
                      runId: liveRun.run_id,
                      action: "reconcile_order",
                    })
                  }
                >
                  <RefreshCw aria-hidden="true" />
                  刷新交易所订单状态
                </button>
              ) : null}
              {liveRun.lifecycle === "market_data_running" &&
              liveRun.execution_order?.client_order_id &&
              liveRun.execution_order_state_sha256 &&
              !liveRun.execution_order.terminal &&
              !liveRun.execution_order.cancel_attempted &&
              liveRunBoundaries?.cancel_order_allowed ? (
                <>
                  <label className={styles.confirmationRow}>
                    <input
                      type="checkbox"
                      checked={cancelConfirmed}
                      onChange={(event) =>
                        setCancelConfirmed(event.target.checked)
                      }
                    />
                    <span>我确认申请撤销当前订单的剩余未成交数量。</span>
                  </label>
                  <button
                    type="button"
                    disabled={
                      !cancelConfirmed || executionCancelOwnerApproval.isPending
                    }
                    onClick={() =>
                      executionCancelOwnerApproval.mutate({
                        runId: liveRun.run_id,
                        request: {
                          run_id: liveRun.run_id,
                          request_id: `cancel-${Date.now()}`,
                          client_order_id:
                            liveRun.execution_order!.client_order_id!,
                          source_order_state_sha256:
                            liveRun.execution_order_state_sha256!,
                          expires_at_unix_ms: Date.now() + 5 * 60 * 1_000,
                          user_confirmed: true,
                        },
                      })
                    }
                  >
                    <Unplug aria-hidden="true" />
                    {executionCancelOwnerApproval.isPending
                      ? "提交中"
                      : "提交人工撤单申请"}
                  </button>
                </>
              ) : null}
              {["created", "preflight_ready", "market_data_running"].includes(
                liveRun.lifecycle,
              ) ? (
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
              {auditAnchorUnavailable
                ? "外部审计锚点尚未配置，Live Run 候选保持阻断。"
                : "先显式检查生产账户。账户连接与交易所交易权限都验证后，才可创建候选。"}
            </p>
            <label className={styles.confirmationRow}>
              <input
                type="checkbox"
                checked={liveRunConfirmed}
                onChange={(event) => setLiveRunConfirmed(event.target.checked)}
              />
              <span>
                我确认创建 Live Run 候选；创建不会自动启动行情或发送订单。
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
