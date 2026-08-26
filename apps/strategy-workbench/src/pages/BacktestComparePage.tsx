import { Link, useNavigate } from "@tanstack/react-router";
import { GitCompareArrows, RefreshCw, ShieldAlert } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import type { Run, RunComparisonItem } from "../api/generated/productApi";
import { productErrorMessage } from "../features/product/presentation";
import {
  useOverviewProductContext,
  useReproduceBacktestRun,
  useRunComparison,
} from "../features/product/useProductResources";
import { ProductErrorState, ProductLoading } from "./ProductState";
import styles from "./Pages.module.css";

const MIN_RUNS = 2;
const MAX_RUNS = 4;

export function BacktestComparePage() {
  const product = useOverviewProductContext();
  const navigate = useNavigate();
  const reproducible = useReproduceBacktestRun();
  const [selectedRunIds, setSelectedRunIds] = useState<string[]>([]);
  const [reproductionSource, setReproductionSource] = useState<string>();
  const [reproductionConfirmed, setReproductionConfirmed] = useState(false);

  const availableRuns = useMemo(
    () => (product.runs?.data ?? []).filter(isComparableRun).slice(0, 20),
    [product.runs?.data],
  );

  useEffect(() => {
    setSelectedRunIds((current) => {
      const available = new Set(availableRuns.map((run) => run.run_id));
      const retained = current.filter((runId) => available.has(runId));
      if (retained.length > 0) return retained;
      return availableRuns.slice(0, MIN_RUNS).map((run) => run.run_id);
    });
  }, [availableRuns]);

  const comparison = useRunComparison(selectedRunIds);
  const reproductionError = reproducible.error
    ? productErrorMessage(reproducible.error)
    : undefined;

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
    return <ProductLoading label="正在验证 Run 列表" />;
  }
  if (product.runtimeError) {
    return (
      <ProductErrorState
        error={product.runtimeError}
        onRetry={product.retryRuns}
        retrying={product.isRuntimeVerifying}
        retryLabel="重新加载 Run"
      />
    );
  }
  if (product.isRuntimeVerifying) {
    return <ProductLoading label="正在验证可比较 Run" />;
  }
  if (!product.runs) {
    return (
      <ProductErrorState
        error={new Error("Run 列表尚未验证")}
        onRetry={product.retryRuns}
        retryLabel="重新加载 Run"
      />
    );
  }

  const toggleRun = (runId: string) => {
    setSelectedRunIds((current) => {
      if (current.includes(runId)) {
        return current.filter((value) => value !== runId);
      }
      return current.length < MAX_RUNS ? [...current, runId] : current;
    });
  };

  const reproduce = () => {
    if (!reproductionSource || !reproductionConfirmed) return;
    reproducible.mutate(reproductionSource, {
      onSuccess: (response) => {
        setReproductionSource(undefined);
        setReproductionConfirmed(false);
        void navigate({
          to: "/runs/$runId",
          params: { runId: response.data.reproduced_run.run_id },
        });
      },
    });
  };

  return (
    <>
      <header className={styles.pageHeading}>
        <div>
          <span className="eyebrow">Run 比较</span>
          <h1>Backtest 与 Demo 行为对比</h1>
          <p>选择 2 至 4 个已冻结 Run；第一项作为比较基准。</p>
        </div>
        <span className={styles.readOnlyBadge}>
          <GitCompareArrows aria-hidden="true" /> 结果只读
        </span>
      </header>

      <section className={styles.comparePicker} aria-label="选择可比较 Run">
        <header>
          <div>
            <span className="eyebrow">Run 范围</span>
            <h2>
              已选择 {selectedRunIds.length} / {MAX_RUNS}
            </h2>
          </div>
          <Link to="/backtests">创建新回测</Link>
        </header>
        {availableRuns.length === 0 ? (
          <p>当前没有具备可信结果产物的 Backtest 或 Demo Run。</p>
        ) : (
          <div className={styles.runSelectorGrid}>
            {availableRuns.map((run) => {
              const selected = selectedRunIds.includes(run.run_id);
              return (
                <label
                  key={run.run_id}
                  className={selected ? styles.runSelected : undefined}
                >
                  <input
                    type="checkbox"
                    checked={selected}
                    disabled={!selected && selectedRunIds.length >= MAX_RUNS}
                    onChange={() => toggleRun(run.run_id)}
                  />
                  <span>
                    <strong>{run.run_id}</strong>
                    <small>
                      {run.strategy_version_id} · {environmentLabel(run)}
                    </small>
                  </span>
                  <em>
                    {selected && selectedRunIds[0] === run.run_id
                      ? "基准"
                      : environmentLabel(run)}
                  </em>
                </label>
              );
            })}
          </div>
        )}
      </section>

      {selectedRunIds.length < MIN_RUNS ? (
        <section className={styles.comparisonNotice}>
          <ShieldAlert aria-hidden="true" />
          <div>
            <strong>至少选择两个 Run</strong>
            <span>接口只接受 2 至 4 个唯一 Run ID。</span>
          </div>
        </section>
      ) : comparison.isPending || comparison.isFetching ? (
        <ProductLoading label="正在校验多 Run 比较结果" />
      ) : comparison.error ? (
        <ProductErrorState
          error={comparison.error}
          onRetry={comparison.refetch}
          retrying={comparison.isFetching}
          retryLabel="重试比较"
        />
      ) : comparison.data ? (
        <>
          <CompatibilityBand
            compatibility={comparison.data.data.compatibility}
          />
          <ComparisonTable items={comparison.data.data.items} />
        </>
      ) : null}

      {reproductionSource ? (
        <section
          className={styles.reproductionPanel}
          aria-label="确定性复现确认"
        >
          <header>
            <div>
              <span className="eyebrow">显式操作</span>
              <h2>复现 {reproductionSource}</h2>
            </div>
            <button
              type="button"
              onClick={() => {
                reproducible.reset();
                setReproductionSource(undefined);
                setReproductionConfirmed(false);
              }}
            >
              取消
            </button>
          </header>
          <p>
            系统将读取不可变请求配置，重新运行真实
            BacktestEngine，并创建一个新的 Run；源 Run 不会被覆盖。
          </p>
          <label>
            <input
              type="checkbox"
              checked={reproductionConfirmed}
              onChange={(event) =>
                setReproductionConfirmed(event.target.checked)
              }
            />
            我确认这是一次用户主动的确定性复现，不是自动重试。
          </label>
          {reproductionError ? (
            <span className={styles.formError} role="alert">
              {`${reproductionError.title}：${reproductionError.detail}。本次不会自动重试，请确认后再次显式提交。`}
            </span>
          ) : null}
          <button
            type="button"
            disabled={!reproductionConfirmed || reproducible.isPending}
            onClick={reproduce}
          >
            <RefreshCw aria-hidden="true" />
            {reproducible.isPending ? "正在复现" : "创建复现 Run"}
          </button>
        </section>
      ) : comparison.data ? (
        <section className={styles.reproductionActions} aria-label="可复现 Run">
          <span>确定性复现</span>
          {comparison.data.data.items.map((item) => {
            const run = availableRuns.find(
              (candidate) => candidate.run_id === item.run_id,
            );
            const canReproduce =
              item.environment === "backtest" &&
              run?.config_ref.startsWith("artifact://backtests/") === true;
            return (
              <button
                key={item.run_id}
                type="button"
                disabled={!canReproduce}
                title={
                  canReproduce
                    ? "重新运行并校验输入输出指纹"
                    : "历史 Run 没有不可变请求文件"
                }
                onClick={() => {
                  reproducible.reset();
                  setReproductionSource(item.run_id);
                  setReproductionConfirmed(false);
                }}
              >
                <RefreshCw aria-hidden="true" /> {item.run_id}
              </button>
            );
          })}
        </section>
      ) : null}
    </>
  );
}

function isComparableRun(run: Run): boolean {
  if (run.environment === "backtest") {
    return (
      run.lifecycle === "completed" &&
      run.result.status === "available" &&
      run.result.result_ref !== null &&
      run.result.report_ref !== null &&
      run.result.analysis_ref !== null
    );
  }
  return run.environment === "sandbox" && run.lifecycle === "stopped";
}

function environmentLabel(run: Pick<Run, "environment">): string {
  return run.environment === "backtest" ? "Backtest" : "Demo";
}

function CompatibilityBand({
  compatibility,
}: {
  compatibility: {
    same_strategy: boolean;
    same_strategy_version: boolean;
    same_parameters: boolean;
    same_data: boolean;
    same_instrument: boolean;
    same_currency: boolean;
    same_environment: boolean;
    behaviorally_comparable: boolean;
    directly_comparable: boolean;
  };
}) {
  return (
    <section
      className={`${styles.comparisonNotice} ${
        compatibility.directly_comparable
          ? styles.comparisonReady
          : styles.comparisonWarning
      }`}
      aria-label="比较兼容性"
    >
      <ShieldAlert aria-hidden="true" />
      <div>
        <strong>
          {compatibility.directly_comparable
            ? "结果可直接比较"
            : compatibility.behaviorally_comparable
              ? "策略行为可比较，数据范围不同"
              : "结果仅可并列查看"}
        </strong>
        <span>
          策略 {yesNo(compatibility.same_strategy)} · 版本{" "}
          {yesNo(compatibility.same_strategy_version)} · 参数{" "}
          {yesNo(compatibility.same_parameters)} · 数据{" "}
          {yesNo(compatibility.same_data)}
          {" · "}标的 {yesNo(compatibility.same_instrument)} · 币种{" "}
          {yesNo(compatibility.same_currency)} · 环境{" "}
          {yesNo(compatibility.same_environment)}
        </span>
      </div>
    </section>
  );
}

function ComparisonTable({ items }: { items: RunComparisonItem[] }) {
  return (
    <section className={styles.panel} aria-label="Run 比较结果">
      <header>
        <div>
          <span className="eyebrow">比较矩阵</span>
          <h2>参数、收益、风险与来源</h2>
        </div>
        <span>{items.length} 个 Run</span>
      </header>
      <div className={styles.comparisonTableWrap}>
        <table className={styles.comparisonTable}>
          <thead>
            <tr>
              <th>指标</th>
              {items.map((item) => (
                <th key={item.run_id}>
                  <Link to="/runs/$runId" params={{ runId: item.run_id }}>
                    {item.run_id}
                  </Link>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            <CompareRow
              label="环境"
              items={items}
              value={(item) =>
                item.environment === "backtest" ? "Backtest" : "Demo"
              }
            />
            <CompareRow
              label="策略版本"
              items={items}
              value={(item) => item.strategy_version_id}
            />
            <CompareRow
              label="数据"
              items={items}
              value={(item) => item.data_ref}
            />
            <CompareRow
              label="标的"
              items={items}
              value={(item) => item.instrument_id}
            />
            <CompareRow
              label="EMA 参数"
              items={items}
              value={(item) =>
                `${item.parameters.fast_period} / ${item.parameters.slow_period}`
              }
            />
            <CompareRow
              label="起始权益"
              items={items}
              value={(item) => item.risk.starting_equity}
            />
            <CompareRow
              label="结束权益"
              items={items}
              value={(item) => item.risk.ending_equity}
            />
            <CompareRow
              label="最大回撤"
              items={items}
              value={(item) => formatRate(item.risk.max_drawdown_rate)}
            />
            <CompareRow
              label="行情 / 成交 / 持仓"
              items={items}
              value={(item) =>
                `${item.metrics.market_event_count} / ${item.metrics.fill_count} / ${item.metrics.position_count}`
              }
            />
            <CompareRow
              label="数据 SHA"
              items={items}
              value={(item) => shortHash(item.data_sha256)}
              mono
            />
            <CompareRow
              label="配置 SHA"
              items={items}
              value={(item) => shortHash(item.config_sha256)}
              mono
            />
          </tbody>
        </table>
      </div>
    </section>
  );
}

function CompareRow({
  label,
  items,
  value,
  mono = false,
}: {
  label: string;
  items: RunComparisonItem[];
  value: (item: RunComparisonItem) => string;
  mono?: boolean;
}) {
  return (
    <tr>
      <th>{label}</th>
      {items.map((item) => (
        <td key={item.run_id} className={mono ? styles.monoCell : undefined}>
          {value(item)}
        </td>
      ))}
    </tr>
  );
}

function formatRate(value: string): string {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? `${(parsed * 100).toFixed(4)}%` : "不可计算";
}

function shortHash(value: string): string {
  return value.length > 24 ? `${value.slice(0, 18)}…${value.slice(-6)}` : value;
}

function yesNo(value: boolean): string {
  return value ? "一致" : "不同";
}
