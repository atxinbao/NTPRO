import {
  Outlet,
  createRootRoute,
  createRoute,
  createRouter,
  redirect,
} from "@tanstack/react-router";

import { AppShell } from "./AppShell";
import { OverviewPage } from "../pages/OverviewPage";
import { BacktestPage } from "../pages/BacktestPage";
import { BacktestComparePage } from "../pages/BacktestComparePage";
import { DemoPage } from "../pages/DemoPage";
import { LivePage } from "../pages/LivePage";
import { RunDetailPage } from "../pages/RunDetailPage";
import { SystemStatusPage } from "../pages/SystemStatusPage";

const rootRoute = createRootRoute({
  component: () => (
    <AppShell>
      <Outlet />
    </AppShell>
  ),
  notFoundComponent: () => <div>页面不存在</div>,
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  beforeLoad: () => {
    throw redirect({ to: "/overview" });
  },
});

const overviewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/overview",
  component: OverviewPage,
});

const backtestRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/backtests",
  component: BacktestPage,
});

const backtestCompareRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/backtests/compare",
  component: BacktestComparePage,
});

const demoRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/demo",
  component: DemoPage,
});

const liveRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/live",
  component: LivePage,
});

const systemStatusRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/system-status",
  component: SystemStatusPage,
});

const runDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/runs/$runId",
  component: RunDetailPage,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  overviewRoute,
  backtestRoute,
  backtestCompareRoute,
  demoRoute,
  liveRoute,
  runDetailRoute,
  systemStatusRoute,
]);

export function createAppRouter() {
  return createRouter({
    routeTree,
    basepath: "/strategy-workbench",
  });
}

export const router = createAppRouter();

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
