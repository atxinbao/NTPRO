import {
  Outlet,
  createRootRoute,
  createRoute,
  createRouter,
  redirect,
} from "@tanstack/react-router";

import { AppShell } from "./AppShell";
import { OverviewPage } from "../pages/OverviewPage";
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

const routeTree = rootRoute.addChildren([
  indexRoute,
  overviewRoute,
  systemStatusRoute,
]);

export const router = createRouter({
  routeTree,
  basepath: "/strategy-workbench",
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
