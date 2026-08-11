import type { LucideIcon } from "lucide-react";
import {
  Activity,
  BookOpenCheck,
  ChevronLeft,
  ChevronRight,
  CircleGauge,
  Database,
  FlaskConical,
  GitCompareArrows,
  LayoutDashboard,
  ListTree,
  Radio,
  RefreshCw,
  ShieldCheck,
} from "lucide-react";
import { Link, useParams, useRouterState } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";

import type { Run, RunEnvironment } from "../api/generated/productApi";
import {
  environmentLabels,
  lifecycleLabels,
  riskLabels,
} from "../features/product/presentation";
import {
  useOverviewProductContext,
  useRunProductContext,
} from "../features/product/useProductResources";
import { useMvpStatus } from "../features/status/useMvpStatus";
import styles from "./AppShell.module.css";

interface AppShellProps {
  children: ReactNode;
}

interface NavigationItem {
  label: string;
  icon: LucideIcon;
  to?:
    | "/overview"
    | "/backtests"
    | "/backtests/compare"
    | "/demo"
    | "/live"
    | "/system-status";
  disabledReason?: string;
}

const navigation: NavigationItem[] = [
  { label: "总览", icon: LayoutDashboard, to: "/overview" },
  {
    label: "策略",
    icon: BookOpenCheck,
    disabledReason: "当前在总览中选择策略",
  },
  { label: "Backtest", icon: FlaskConical, to: "/backtests" },
  { label: "运行对比", icon: GitCompareArrows, to: "/backtests/compare" },
  { label: "Demo", icon: Radio, to: "/demo" },
  { label: "Live", icon: Activity, to: "/live" },
  { label: "运行", icon: ListTree, disabledReason: "从策略总览进入 Run 详情" },
  { label: "数据", icon: Database, disabledReason: "等待数据产品合同" },
  { label: "风险", icon: ShieldCheck, disabledReason: "等待风险产品合同" },
  { label: "系统状态", icon: CircleGauge, to: "/system-status" },
];

type DockTab = "positions" | "activity" | "fills" | "logs";

const dockTabs: Array<{ id: DockTab; label: string }> = [
  { id: "positions", label: "持仓" },
  { id: "activity", label: "活动" },
  { id: "fills", label: "成交" },
  { id: "logs", label: "日志" },
];

function initialDrawerState(): boolean {
  return (
    typeof window !== "undefined" &&
    window.matchMedia?.("(min-width: 761px)").matches === true
  );
}

export function AppShell({ children }: AppShellProps) {
  const queryClient = useQueryClient();
  const query = useMvpStatus();
  const data = query.error ? undefined : query.data;
  const params = useParams({ strict: false });
  const routePath = useRouterState({
    select: (state) => state.location.pathname,
  });
  const isBacktestRoute = routePath.endsWith("/backtests");
  const routeRunId =
    "runId" in params && typeof params.runId === "string"
      ? params.runId
      : undefined;
  const overviewProduct = useOverviewProductContext();
  const runProduct = useRunProductContext(routeRunId);
  const product = routeRunId ? runProduct : overviewProduct;
  const selectedStrategy = product.isReady ? product.strategy : undefined;
  const selectedVersion = product.isReady ? product.version : undefined;
  const runItems = routeRunId
    ? runProduct.isReady && runProduct.run
      ? [runProduct.run]
      : []
    : overviewProduct.isReady
      ? (overviewProduct.runs?.data ?? [])
      : [];
  const overviewRun = isBacktestRoute
    ? (runItems.find((run) => run.environment === "backtest") ?? runItems[0])
    : (runItems.find((run) => run.lifecycle === "running") ?? runItems[0]);
  const currentRun = routeRunId
    ? runProduct.isReady
      ? runProduct.run
      : undefined
    : overviewRun;
  const [drawerOpen, setDrawerOpen] = useState(initialDrawerState);
  const [dockTab, setDockTab] = useState<DockTab>("positions");
  const health = data?.axes.technicalHealth.status;
  const technicalReady = health === "healthy";
  const productReady = product.isReady;
  const combinedReady = technicalReady && productReady;

  const refreshAll = () => {
    void Promise.all([
      queryClient.resetQueries({ queryKey: ["product"] }),
      query.refetch(),
    ]);
  };

  return (
    <div
      className={`${styles.shell} ${drawerOpen ? styles.drawerOpen : ""}`}
      data-testid="app-shell"
    >
      <aside className={styles.rail} aria-label="策略工作台导航">
        <div className={styles.brand}>
          <span className={styles.brandMark}>NT</span>
          <div>
            <strong>NTPRO</strong>
            <small>策略工作台</small>
          </div>
        </div>
        <nav className={styles.navigation}>
          {navigation.map(({ label, icon: Icon, to, disabledReason }) =>
            to ? (
              <Link
                key={label}
                to={to}
                className={styles.navItem}
                activeProps={{
                  className: `${styles.navItem} ${styles.navItemActive}`,
                }}
              >
                <Icon aria-hidden="true" />
                <span>{label}</span>
              </Link>
            ) : (
              <button
                key={label}
                type="button"
                className={`${styles.navItem} ${styles.navItemDisabled}`}
                disabled
                title={disabledReason}
              >
                <Icon aria-hidden="true" />
                <span>{label}</span>
                {label === "Live" ? <em>未开放</em> : null}
              </button>
            ),
          )}
        </nav>
        <div className={styles.railStatus}>
          <span
            className={`${styles.statusDot} ${technicalReady ? styles.ready : styles.blocked}`}
          />
          <div>
            <strong>
              {query.isPending ? "读取中" : data ? "只读连接" : "连接阻断"}
            </strong>
            <small>Live 权限关闭</small>
          </div>
        </div>
      </aside>

      <section className={styles.stage}>
        <header className={styles.topbar}>
          <Scope
            label="策略"
            value={
              currentRun?.strategy_id ??
              selectedStrategy?.strategy_id ??
              "策略未加载"
            }
            testId="strategy-name"
          />
          <Scope
            label="版本"
            value={
              currentRun?.strategy_version_id ??
              selectedVersion?.strategy_version_id ??
              "未知"
            }
          />
          <Scope
            label="模式"
            value={
              currentRun ? environmentLabels[currentRun.environment] : "未知"
            }
          />
          <Scope
            label="账户"
            value={currentRun?.account_ref ?? "未知"}
            optional
          />
          <Scope
            label="Venue"
            value={currentRun?.venue_ref ?? "未知"}
            optional
          />
          <div className={styles.topActions}>
            <span
              className={`${styles.healthChip} ${combinedReady ? styles.healthReady : styles.healthBlocked}`}
            >
              {product.isVerifying
                ? "产品验证中"
                : product.error
                  ? "产品阻断"
                  : query.isPending || query.isFetching
                    ? "技术连接中"
                    : data && productReady
                      ? "只读就绪"
                      : "技术阻断"}
            </span>
            <button
              type="button"
              className={styles.iconButton}
              title="刷新产品与系统状态"
              aria-label="刷新产品与系统状态"
              disabled={query.isFetching || product.isVerifying}
              onClick={refreshAll}
            >
              <RefreshCw aria-hidden="true" />
            </button>
          </div>
        </header>

        <div className={styles.modeTabs} aria-label="策略运行模式">
          <Mode
            label="Backtest"
            detail={modeDetail(runItems, "backtest")}
            state={modeState(runItems, "backtest")}
          />
          <Mode
            label="Demo"
            detail={modeDetail(runItems, "sandbox")}
            state={modeState(runItems, "sandbox")}
          />
          <Mode
            label="Live"
            detail={modeDetail(runItems, "live")}
            state={modeState(runItems, "live")}
          />
          <Mode label="运行对比" detail="2 至 4 个 Backtest" state="complete" />
        </div>

        <div className={styles.workarea}>
          <main className={styles.canvas}>{children}</main>
          <aside className={styles.inspector} aria-label="当前运行详情">
            <button
              type="button"
              className={styles.drawerToggle}
              title={drawerOpen ? "收起详情栏" : "展开详情栏"}
              aria-label={drawerOpen ? "收起详情栏" : "展开详情栏"}
              aria-expanded={drawerOpen}
              onClick={() => setDrawerOpen((open) => !open)}
            >
              {drawerOpen ? <ChevronRight /> : <ChevronLeft />}
            </button>
            <div className={styles.inspectorBody}>
              <header>
                <span className="eyebrow">当前选择</span>
                <strong>{currentRun?.run_id ?? "Run 未加载"}</strong>
              </header>
              <div className={styles.kvList}>
                <KeyValue
                  label="环境"
                  value={
                    currentRun
                      ? environmentLabels[currentRun.environment]
                      : "未知"
                  }
                />
                <KeyValue
                  label="账户"
                  value={currentRun?.account_ref ?? "未知"}
                />
                <KeyValue
                  label="Venue"
                  value={currentRun?.venue_ref ?? "未知"}
                />
                <KeyValue label="节点" value={data?.nodeId ?? "未知"} />
              </div>
              <section>
                <span className="eyebrow">准入边界</span>
                <div className={styles.boundaryList}>
                  <strong>只读 Product API</strong>
                  <span>外部 Venue：关闭</span>
                  <span>订单提交与修改：关闭</span>
                  <span>自动重试与补救：关闭</span>
                </div>
              </section>
              <section>
                <span className="eyebrow">数据来源</span>
                <div className={styles.sourceList}>
                  {(currentRun?.source.source_refs ?? []).map((source) => (
                    <span key={source}>{source}</span>
                  ))}
                  {!currentRun ? <span>等待产品资源验证</span> : null}
                </div>
              </section>
            </div>
          </aside>
        </div>

        <section className={styles.bottomDock} aria-label="策略运行活动区">
          <div className={styles.dockTabs} role="tablist">
            {dockTabs.map((tab) => (
              <button
                key={tab.id}
                type="button"
                role="tab"
                aria-selected={dockTab === tab.id}
                className={dockTab === tab.id ? styles.dockTabActive : ""}
                onClick={() => setDockTab(tab.id)}
              >
                {tab.label}
              </button>
            ))}
          </div>
          <DockContent tab={dockTab} run={currentRun} />
        </section>

        <footer className={styles.statusbar}>
          <StatusItem
            label="数据"
            value={currentRun?.source.freshness_status ?? "未知"}
            warning={!currentRun}
          />
          <StatusItem label="账户" value={currentRun?.account_ref ?? "未知"} />
          <StatusItem label="Venue" value={currentRun?.venue_ref ?? "未知"} />
          <StatusItem
            label="风险"
            value={currentRun ? riskLabels[currentRun.risk.status] : "未知"}
            warning={!currentRun || currentRun.risk.status === "blocked"}
          />
          <StatusItem label="节点" value={data?.nodeId ?? "未知"} />
          <StatusItem
            label="更新"
            value={
              currentRun
                ? new Date(currentRun.updated_at_unix_ms).toLocaleTimeString(
                    "zh-CN",
                  )
                : "未知"
            }
          />
        </footer>
      </section>
    </div>
  );
}

function Scope({
  label,
  value,
  optional,
  testId,
}: {
  label: string;
  value: string;
  optional?: boolean;
  testId?: string;
}) {
  return (
    <div className={`${styles.scope} ${optional ? styles.scopeOptional : ""}`}>
      <span>{label}</span>
      <strong data-testid={testId}>{value}</strong>
    </div>
  );
}

function Mode({
  label,
  detail,
  state,
}: {
  label: string;
  detail: string;
  state: "complete" | "active" | "blocked" | "neutral";
}) {
  return (
    <button
      type="button"
      className={state === "active" ? styles.modeActive : ""}
      disabled
    >
      <span className={`${styles.modeDot} ${styles[state]}`} />
      <strong>{label}</strong>
      <small>{detail}</small>
    </button>
  );
}

function KeyValue({ label, value }: { label: string; value: string }) {
  return (
    <div className={styles.kvRow}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function StatusItem({
  label,
  value,
  warning,
}: {
  label: string;
  value: string;
  warning?: boolean;
}) {
  return (
    <span className={warning ? styles.statusWarning : ""}>
      {label}：{value}
    </span>
  );
}

function DockContent({ tab, run }: { tab: DockTab; run?: Run }) {
  const content: Record<
    DockTab,
    { title: string; value: string; note: string }
  > = {
    positions: {
      title: "当前持仓",
      value: "未接入",
      note: run ? "Run 持仓产品合同待建设" : "Run 未选择",
    },
    activity: {
      title: "运行活动",
      value: run ? lifecycleLabels[run.lifecycle] : "Run 未选择",
      note: run?.error?.summary ?? "当前生命周期快照",
    },
    fills: {
      title: "当前成交",
      value: "未接入",
      note: run ? "Run 成交产品合同待建设" : "Run 未选择",
    },
    logs: {
      title: "日志摘要",
      value: run?.error?.summary ?? "无产品错误",
      note: "原始技术日志不在主产品面暴露",
    },
  };
  const selected = content[tab];
  return (
    <div className={styles.dockContent}>
      <article>
        <span>{selected.title}</span>
        <strong>{selected.value}</strong>
        <small>{selected.note}</small>
      </article>
      <article>
        <span>Run</span>
        <strong>{run?.run_id ?? "未加载"}</strong>
        <small>当前只读产品资源</small>
      </article>
      <article>
        <span>风险状态</span>
        <strong>{run ? riskLabels[run.risk.status] : "未知"}</strong>
        <small>{run?.risk.risk_ref ?? "等待 Run"}</small>
      </article>
    </div>
  );
}

function modeDetail(runs: Run[], environment: RunEnvironment): string {
  const run = runs.find((item) => item.environment === environment);
  return run ? lifecycleLabels[run.lifecycle] : "暂无 Run";
}

function modeState(
  runs: Run[],
  environment: RunEnvironment,
): "complete" | "active" | "blocked" | "neutral" {
  const run = runs.find((item) => item.environment === environment);
  if (!run) return "neutral";
  if (
    run.risk.status === "blocked" ||
    ["failed", "cancelled", "stopped"].includes(run.lifecycle)
  ) {
    return "blocked";
  }
  if (run.lifecycle === "running") return "active";
  if (run.lifecycle === "completed") return "complete";
  return "neutral";
}
