import { Link, useNavigate } from "@tanstack/react-router";
import { Play, Radio, ShieldCheck } from "lucide-react";
import { useState, type FormEvent } from "react";

import type { CreateDemoRunRequest } from "../api/generated/productApi";
import { productErrorMessage } from "../features/product/presentation";
import {
  useCreateDemoRun,
  useOverviewProductContext,
} from "../features/product/useProductResources";
import { useMvpStatus } from "../features/status/useMvpStatus";
import { ProductErrorState, ProductLoading } from "./ProductState";
import styles from "./Pages.module.css";

const STATIONARY_STOPPED_RUNTIME_REASONS = new Set([
  "supervisor_process_not_running",
  "node_status_timestamp_marked_stale",
]);

function isDemoNodeReady(runtime: {
  status: string;
  availability: string;
  freshness: string;
  reasons: string[];
  error?: string;
}) {
  const stationaryStoppedState =
    runtime.freshness === "stale" &&
    runtime.reasons.length === STATIONARY_STOPPED_RUNTIME_REASONS.size &&
    runtime.reasons.every((reason) =>
      STATIONARY_STOPPED_RUNTIME_REASONS.has(reason),
    );

  return (
    runtime.status === "stopped" &&
    runtime.availability === "available" &&
    !runtime.error &&
    (runtime.freshness === "fresh" || stationaryStoppedState)
  );
}

export function DemoPage() {
  const product = useOverviewProductContext();
  const status = useMvpStatus();
  const createRun = useCreateDemoRun();
  const navigate = useNavigate();
  const [confirmed, setConfirmed] = useState(false);
  const [formError, setFormError] = useState<string>();

  if (product.error) {
    return (
      <ProductErrorState
        error={product.error}
        onRetry={product.retryProduct}
        retrying={product.isVerifying}
        retryLabel="重新验证策略"
      />
    );
  }
  if (product.isVerifying || !product.isReady) {
    return <ProductLoading label="正在验证 Demo 创建上下文" />;
  }
  if (status.error) {
    return (
      <ProductErrorState
        error={status.error}
        onRetry={status.refetch}
        retrying={status.isFetching}
        retryLabel="重新验证节点状态"
      />
    );
  }
  if (status.isPending || !status.data) {
    return <ProductLoading label="正在验证 Sandbox 节点" />;
  }
  if (status.isFetching) {
    return <ProductLoading label="正在验证 Sandbox 节点" />;
  }
  if (!product.strategy || !product.version) {
    return <ProductErrorState error={new Error("当前没有可用策略版本")} />;
  }
  if (product.runtimeError) {
    return (
      <ProductErrorState
        error={product.runtimeError}
        onRetry={product.retryRuns}
        retrying={product.isRuntimeVerifying}
        retryLabel="重新加载 Demo Run"
      />
    );
  }
  if (product.isRuntimeVerifying) {
    return <ProductLoading label="正在验证现有 Demo Run" />;
  }
  if (!product.runs) {
    return (
      <ProductErrorState
        error={new Error("Demo Run 列表尚未验证")}
        onRetry={product.retryRuns}
        retryLabel="重新加载 Demo Run"
      />
    );
  }

  const existing = product.runs?.data.find(
    (run) =>
      run.environment === "sandbox" &&
      !["stopped", "failed"].includes(run.lifecycle),
  );
  const identityMatches =
    status.data.strategyId === product.strategy.strategy_id &&
    status.data.strategyVersion === product.version.version;
  const runtime = status.data.axes.runtime;
  const nodeReadyForDemo = isDemoNodeReady(runtime);

  if (!existing && !nodeReadyForDemo) {
    return (
      <ProductErrorState
        error={
          new Error(
            "Sandbox 节点尚未处于可创建状态，需要状态可用、运行状态为 stopped，并且来源新鲜或明确为已停止的静态节点。",
          )
        }
        onRetry={status.refetch}
        retrying={status.isFetching}
        retryLabel="重新验证节点状态"
      />
    );
  }

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setFormError(undefined);
    if (
      !identityMatches ||
      !nodeReadyForDemo ||
      status.isFetching ||
      !confirmed
    ) {
      setFormError("请先确认策略版本与 Sandbox 节点绑定。 ");
      return;
    }
    const request: CreateDemoRunRequest = {
      strategy_id: product.strategy!.strategy_id,
      strategy_version_id: product.version!.strategy_version_id,
      environment: "sandbox",
      supervisor_node_id: status.data!.nodeId,
      account_ref: `account://sandbox/${status.data!.accountId}`,
      venue_ref: `venue://sandbox/${status.data!.venueId}`,
      user_confirmed: true,
    };
    createRun.mutate(request, {
      onSuccess: (response) => {
        void navigate({
          to: "/runs/$runId",
          params: { runId: response.data.run_id },
        });
      },
      onError: (requestError) => {
        const message = productErrorMessage(requestError);
        setFormError(
          `${message.title}：${message.detail}。本次不会自动重试，请确认后再次提交。`,
        );
      },
    });
  };

  return (
    <>
      <header className={styles.pageHeading}>
        <div>
          <span className="eyebrow">Demo</span>
          <h1>Sandbox 策略运行</h1>
          <p>
            复用当前不可变策略版本，绑定 Supervisor 管理的单一 Sandbox 节点。
          </p>
        </div>
        <span className={styles.readOnlyBadge}>
          <ShieldCheck aria-hidden="true" /> 真实订单关闭
        </span>
      </header>

      {existing ? (
        <section className={styles.panel} aria-label="当前 Demo Run">
          <header>
            <div>
              <span className="eyebrow">当前 Run</span>
              <h2>{existing.run_id}</h2>
            </div>
            <span>{existing.lifecycle}</span>
          </header>
          <div className={styles.versionSummary}>
            <ReadOnlyField
              label="策略版本"
              value={existing.strategy_version_id}
            />
            <ReadOnlyField
              label="Supervisor 节点"
              value={existing.runtime?.supervisor_node_id ?? "状态不可验证"}
            />
            <ReadOnlyField label="账户" value={existing.account_ref} />
            <ReadOnlyField label="Venue" value={existing.venue_ref} />
          </div>
          <div className={styles.runActions}>
            <span>同一时间只允许一个 Demo Run。</span>
            <Link to="/runs/$runId" params={{ runId: existing.run_id }}>
              查看运行详情
            </Link>
          </div>
        </section>
      ) : (
        <section className={styles.backtestLayout}>
          <form className={styles.backtestForm} onSubmit={submit}>
            <header>
              <div>
                <span className="eyebrow">运行绑定</span>
                <h2>{product.version.strategy_version_id}</h2>
              </div>
              <Radio aria-hidden="true" />
            </header>
            <div className={styles.formGrid}>
              <ReadOnlyField
                label="策略"
                value={product.strategy.strategy_id}
              />
              <ReadOnlyField label="版本" value={product.version.version} />
              <ReadOnlyField
                label="Supervisor 节点"
                value={status.data.nodeId}
                wide
              />
              <ReadOnlyField
                label="策略实例"
                value={status.data.strategyInstanceId}
                wide
              />
              <ReadOnlyField
                label="Sandbox 账户"
                value={status.data.accountId}
              />
              <ReadOnlyField
                label="Sandbox Venue"
                value={status.data.venueId}
              />
            </div>
            <label className={styles.confirmationRow}>
              <input
                type="checkbox"
                checked={confirmed}
                disabled={!nodeReadyForDemo || status.isFetching}
                onChange={(event) => setConfirmed(event.target.checked)}
              />
              <span>
                我确认创建 Demo Run，并绑定上述策略版本和 Sandbox 节点。
              </span>
            </label>
            {!identityMatches ? (
              <div className={styles.formError} role="alert">
                共享状态与当前策略版本不一致，已阻止创建。
              </div>
            ) : formError ? (
              <div className={styles.formError} role="alert">
                {formError}
              </div>
            ) : null}
            <footer>
              <div>
                <strong>创建后再显式启动</strong>
                <span>创建不会连接外部 Venue，也不会提交真实订单。</span>
              </div>
              <button
                type="submit"
                disabled={
                  !identityMatches ||
                  !nodeReadyForDemo ||
                  status.isFetching ||
                  !confirmed ||
                  createRun.isPending
                }
              >
                <Play aria-hidden="true" />
                {createRun.isPending ? "正在创建" : "创建 Demo Run"}
              </button>
            </footer>
          </form>

          <aside className={styles.backtestBoundary}>
            <span className="eyebrow">能力边界</span>
            <h2>只启用 Sandbox 生命周期</h2>
            <Boundary label="Demo 创建" enabled />
            <Boundary label="Supervisor 启停" enabled />
            <Boundary label="外部 Venue 连接" />
            <Boundary label="真实订单提交" />
            <Boundary label="自动重试与补救" />
            <Boundary label="Live 创建" />
          </aside>
        </section>
      )}
    </>
  );
}

function ReadOnlyField({
  label,
  value,
  wide = false,
}: {
  label: string;
  value: string;
  wide?: boolean;
}) {
  return (
    <label className={wide ? styles.formFieldWide : undefined}>
      <span>{label}</span>
      <input value={value} readOnly />
    </label>
  );
}

function Boundary({
  label,
  enabled = false,
}: {
  label: string;
  enabled?: boolean;
}) {
  return (
    <div>
      <span>{label}</span>
      <strong className={enabled ? styles.boundaryEnabled : undefined}>
        {enabled ? "启用" : "关闭"}
      </strong>
    </div>
  );
}
