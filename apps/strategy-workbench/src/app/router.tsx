import {
  Outlet,
  createRootRoute,
  createRoute,
  createRouter,
  redirect,
} from "@tanstack/react-router";

import { AppShell } from "./AppShell";
import { OverviewPage } from "../pages/OverviewPage";
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
