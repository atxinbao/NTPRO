import { useNavigate } from "@tanstack/react-router";
import { FlaskConical, Play, ShieldCheck } from "lucide-react";
import { useState, type FormEvent } from "react";

import type {
  CreateBacktestRunRequest,
  Run,
  StrategyVersion,
} from "../api/generated/productApi";
import {
  useCreateBacktestRun,
  useOverviewProductContext,
} from "../features/product/useProductResources";
import { ProductErrorState, ProductLoading } from "./ProductState";
import styles from "./Pages.module.css";

export function BacktestPage() {
  const product = useOverviewProductContext();
  const createRun = useCreateBacktestRun();
  const navigate = useNavigate();
  const [formError, setFormError] = useState<string>();

  if (product.error) return <ProductErrorState error={product.error} />;
  if (product.isVerifying || !product.isReady) {
    return <ProductLoading label="正在验证 Backtest 创建上下文" />;
  }
  if (product.runtimeError) {
    return <ProductErrorState error={product.runtimeError} />;
  }
  if (product.isRuntimeVerifying) {
    return <ProductLoading label="正在验证 Backtest 运行数据" />;
  }
  if (!product.strategy || !product.version) {
    return <ProductErrorState error={new Error("当前没有可用策略版本")} />;
  }
  if (!product.runs) {
    return <ProductErrorState error={new Error("Backtest 运行列表尚未验证")} />;
  }

  const baseline = product.runs?.data.find(
    (run) => run.environment === "backtest",
  );
  const source = resolveBacktestSource(
    product.strategy.strategy_id,
    product.version,
    baseline,
  );
  if (!source) {
    return (
      <ProductErrorState
        error={new Error("当前策略版本没有唯一的内置回测来源")}
      />
    );
  }
  const fastPeriod = parameterConst(
    product.version.parameter_schema,
    "fast_period",
  );
  const slowPeriod = parameterConst(
    product.version.parameter_schema,
    "slow_period",
  );
  const canCreate = fastPeriod !== undefined && slowPeriod !== undefined;

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setFormError(undefined);
    if (!canCreate) {
      setFormError("当前策略版本缺少固定 EMA 参数，不能创建回测。");
      return;
    }
    const form = new FormData(event.currentTarget);
    const request: CreateBacktestRunRequest = {
      strategy_id: product.strategy!.strategy_id,
      strategy_version_id: product.version!.strategy_version_id,
      environment: "backtest",
      data_ref: source.dataRef,
      venue_ref: source.venueRef,
      starting_balance: String(form.get("starting_balance")),
      quotes: Number(form.get("quotes")),
      trade_size: String(form.get("trade_size")),
      fast_period: fastPeriod,
      slow_period: slowPeriod,
    };
    createRun.mutate(request, {
      onSuccess: (response) => {
        void navigate({
          to: "/runs/$runId",
          params: { runId: response.data.run_id },
        });
      },
      onError: (error) => setFormError(error.message),
    });
  };

  return (
    <>
      <header className={styles.pageHeading}>
        <div>
          <span className="eyebrow">Backtest</span>
          <h1>创建策略回测</h1>
          <p>使用当前不可变策略版本与确定性数据运行真实 BacktestEngine。</p>
        </div>
        <span className={styles.readOnlyBadge}>
          <ShieldCheck aria-hidden="true" /> 仅模拟成交
        </span>
      </header>

      <section className={styles.backtestLayout}>
        <form className={styles.backtestForm} onSubmit={submit}>
          <header>
            <div>
              <span className="eyebrow">运行配置</span>
              <h2>{product.version.strategy_version_id}</h2>
            </div>
            <FlaskConical aria-hidden="true" />
          </header>

          {source.isBuiltinFallback ? (
            <div className={styles.connectionBanner} role="status">
              <div>
                <strong>当前没有历史 Backtest</strong>
                <span>
                  本次使用策略版本登记的内置确定性数据，只验证产品流程和可复现性，不代表真实市场研究或收益证明。
                </span>
              </div>
              <em>内置数据</em>
            </div>
          ) : null}

          <div className={styles.formGrid}>
            <ReadOnlyField label="策略" value={product.strategy.strategy_id} />
            <ReadOnlyField label="版本" value={product.version.version} />
            <ReadOnlyField label="回测数据" value={source.dataRef} wide />
            <ReadOnlyField label="模拟 Venue" value={source.venueRef} wide />
            <label>
              <span>初始资金</span>
              <input
                name="starting_balance"
                defaultValue="1000000 USDT"
                pattern="[0-9][0-9_]*(\.[0-9]+)? USDT"
                required
              />
            </label>
            <label>
              <span>行情条数</span>
              <input
                name="quotes"
                type="number"
                defaultValue="120"
                min="30"
                max="10000"
                step="1"
                required
              />
            </label>
            <label>
              <span>每次交易数量</span>
              <input
                name="trade_size"
                defaultValue="0.001000"
                pattern="[0-9]+\.[0-9]{6}"
                required
              />
            </label>
            <ReadOnlyField
              label="EMA 周期"
              value={
                canCreate ? `${fastPeriod} / ${slowPeriod}` : "版本参数不可用"
              }
            />
          </div>

          {formError ? (
            <div className={styles.formError} role="alert">
              {formError}
            </div>
          ) : null}

          <footer>
            <div>
              <strong>创建后不可覆盖</strong>
              <span>配置、输入摘要与结果均绑定 SHA-256。</span>
            </div>
            <button type="submit" disabled={!canCreate || createRun.isPending}>
              <Play aria-hidden="true" />
              {createRun.isPending ? "正在运行回测" : "创建并运行"}
            </button>
          </footer>
        </form>

        <aside className={styles.backtestBoundary}>
          <span className="eyebrow">能力边界</span>
          <h2>本次只创建 Backtest</h2>
          <Boundary label="真实 BacktestEngine" enabled />
          <Boundary label="模拟 Venue" enabled />
          <Boundary label="外部 Venue 连接" />
          <Boundary label="真实订单提交" />
          <Boundary label="自动重试与补救" />
          <Boundary label="Demo / Live 创建" />
        </aside>
      </section>
    </>
  );
}

function resolveBacktestSource(
  strategyId: string,
  version: StrategyVersion,
  baseline?: Run,
) {
  if (baseline) {
    return {
      dataRef: baseline.data_ref,
      venueRef: baseline.venue_ref,
      isBuiltinFallback: false,
    };
  }

  const requirements = version.data_requirements;
  if (
    requirements.deterministic_replay_required !== true ||
    requirements.venues.length !== 1 ||
    requirements.symbols.length !== 1
  ) {
    return undefined;
  }

  return {
    dataRef: `dataset://fixtures/${strategyId.replaceAll("_", "-")}`,
    venueRef: `venue://simulated/${requirements.venues[0]}`,
    isBuiltinFallback: true,
  };
}

function parameterConst(schema: Record<string, unknown>, name: string) {
  const properties = schema.properties;
  if (!properties || typeof properties !== "object") return undefined;
  const property = (properties as Record<string, unknown>)[name];
  if (!property || typeof property !== "object") return undefined;
  const value = (property as Record<string, unknown>).const;
  return typeof value === "number" && Number.isSafeInteger(value)
    ? value
    : undefined;
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
