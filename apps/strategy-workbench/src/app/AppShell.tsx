import type { LucideIcon } from "lucide-react";
import {
  Activity,
  BookOpenCheck,
  ChevronLeft,
  ChevronRight,
  CircleGauge,
  Database,
  FlaskConical,
  LayoutDashboard,
  ListTree,
  Radio,
  RefreshCw,
  ShieldCheck,
} from "lucide-react";
import { Link } from "@tanstack/react-router";
import { useState, type ReactNode } from "react";

import { statusLabel } from "../api/mvpStatus";
import { useMvpStatus } from "../features/status/useMvpStatus";
import styles from "./AppShell.module.css";

interface AppShellProps {
  children: ReactNode;
}

interface NavigationItem {
  label: string;
  icon: LucideIcon;
  to?: "/overview" | "/system-status";
  disabledReason?: string;
}

const navigation: NavigationItem[] = [
  { label: "总览", icon: LayoutDashboard, to: "/overview" },
  {
    label: "策略",
    icon: BookOpenCheck,
    disabledReason: "等待 S0 产品资源合同",
  },
  { label: "Backtest", icon: FlaskConical, disabledReason: "等待 S1 产品化" },
  { label: "Demo", icon: Radio, disabledReason: "等待 S2 产品化" },
  { label: "Live", icon: Activity, disabledReason: "等待 S3 独立准入" },
  { label: "运行", icon: ListTree, disabledReason: "等待 Run 产品合同" },
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
  const query = useMvpStatus();
  const data = query.error ? undefined : query.data;
  const [drawerOpen, setDrawerOpen] = useState(initialDrawerState);
  const [dockTab, setDockTab] = useState<DockTab>("positions");
  const health = data?.axes.technicalHealth.status;
  const ready = health === "healthy";

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
            className={`${styles.statusDot} ${ready ? styles.ready : styles.blocked}`}
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
            value={data?.strategyId ?? "策略未加载"}
            testId="strategy-name"
          />
          <Scope label="版本" value={data?.strategyVersion ?? "未知"} />
          <Scope label="模式" value="Demo / Sandbox" />
          <Scope label="账户" value={data?.accountId ?? "未知"} optional />
          <Scope label="Venue" value={data?.venueId ?? "未知"} optional />
          <div className={styles.topActions}>
            <span
              className={`${styles.healthChip} ${ready ? styles.healthReady : styles.healthBlocked}`}
            >
              {query.isPending
                ? "正在连接"
                : data
                  ? statusLabel(health ?? "")
                  : "合同阻断"}
            </span>
            <button
              type="button"
              className={styles.iconButton}
              title="刷新共享状态"
              aria-label="刷新共享状态"
              disabled={query.isFetching}
              onClick={() => void query.refetch()}
            >
              <RefreshCw aria-hidden="true" />
            </button>
          </div>
        </header>

        <div className={styles.modeTabs} aria-label="策略运行模式">
          <Mode label="Backtest" detail="历史引用" state="complete" />
          <Mode
            label="Demo"
            detail={data ? statusLabel(data.axes.runtime.status) : "读取中"}
            state="active"
          />
          <Mode label="Live" detail="未开放" state="blocked" />
          <Mode label="运行对比" detail="等待产品合同" state="neutral" />
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
                <strong>{data?.strategyInstanceId ?? "Run 未加载"}</strong>
              </header>
              <div className={styles.kvList}>
                <KeyValue label="环境" value="Demo / Sandbox" />
                <KeyValue label="账户" value={data?.accountId ?? "未知"} />
                <KeyValue label="Venue" value={data?.venueId ?? "未知"} />
                <KeyValue label="节点" value={data?.nodeId ?? "未知"} />
              </div>
              <section>
                <span className="eyebrow">准入边界</span>
                <div className={styles.boundaryList}>
                  <strong>只读状态桥接</strong>
                  <span>真实 Venue：关闭</span>
                  <span>订单提交与修改：关闭</span>
                  <span>自动重试与补救：关闭</span>
                </div>
              </section>
              <section>
                <span className="eyebrow">数据来源</span>
                <div className={styles.sourceList}>
                  {(data?.sourceRefs ?? []).map((source) => (
                    <span key={source}>{source}</span>
                  ))}
                  {!data ? <span>等待验证</span> : null}
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
          <DockContent tab={dockTab} data={data} />
        </section>

        <footer className={styles.statusbar}>
          <StatusItem
            label="数据"
            value={data?.business.freshness ?? "未知"}
            warning={!data}
          />
          <StatusItem label="账户" value={data?.accountId ?? "未知"} />
          <StatusItem label="Venue" value={data?.venueId ?? "未知"} />
          <StatusItem label="风险" value="阻断" warning />
          <StatusItem label="节点" value={data?.nodeId ?? "未知"} />
          <StatusItem
            label="更新"
            value={
              data
                ? new Date(data.generatedAtUnixMs).toLocaleTimeString("zh-CN")
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

function DockContent({
  tab,
  data,
}: {
  tab: DockTab;
  data: ReturnType<typeof useMvpStatus>["data"];
}) {
  const content: Record<
    DockTab,
    { title: string; value: string; note: string }
  > = {
    positions: {
      title: "当前持仓",
      value: data?.business.positions ?? "共享状态不可用",
      note: "产品明细合同待建设",
    },
    activity: {
      title: "运行活动",
      value: data?.business.lifecycle ?? "共享状态不可用",
      note: data?.axes.runtime.status
        ? statusLabel(data.axes.runtime.status)
        : "状态未知",
    },
    fills: {
      title: "当前成交",
      value: data?.business.fills ?? "共享状态不可用",
      note: "真实成交能力关闭",
    },
    logs: {
      title: "日志摘要",
      value: data?.business.diagnostic ?? "共享状态不可用",
      note: "原始日志不在主产品面暴露",
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
        <strong>{data?.strategyInstanceId ?? "未加载"}</strong>
        <small>当前只读实例</small>
      </article>
      <article>
        <span>风险状态</span>
        <strong>阻断</strong>
        <small>Live 权限未开放</small>
      </article>
    </div>
  );
}
