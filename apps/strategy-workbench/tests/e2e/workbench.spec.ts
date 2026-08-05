import { expect, test } from "@playwright/test";

import { validStatusPayload } from "../../src/test/fixtures";

test.beforeEach(async ({ page }) => {
  await page.route("**/api/mvp/v1/status", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(validStatusPayload),
    });
  });
});

test("desktop shell renders verified read-only status", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("overview");
  await expect(page.getByText("策略状态已验证")).toBeVisible();
  await expect(page.getByTestId("strategy-name")).toHaveText("btc-ema");
  for (const liveButton of await page
    .getByRole("button", { name: /Live/ })
    .all()) {
    await expect(liveButton).toBeDisabled();
  }
  await expect(
    page.getByRole("button", { name: /下单|撤单|改单|平仓/ }),
  ).toHaveCount(0);
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: testInfo.outputPath("strategy-workbench-1440.png"),
    fullPage: true,
  });

  await page.getByRole("link", { name: "系统状态" }).click();
  await expect(page.getByRole("heading", { name: "系统状态" })).toBeVisible();
});

test("mobile shell keeps the drawer closed and has no page overflow", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("overview");
  await expect(page.getByText("策略状态已验证")).toBeVisible();
  await expect(page.getByTestId("app-shell")).not.toHaveClass(/drawerOpen/);
  const layout = await page.evaluate(() => ({
    documentWidth: document.documentElement.scrollWidth,
    viewportWidth: document.documentElement.clientWidth,
    scrollX: window.scrollX,
  }));
  expect(layout.documentWidth).toBeLessThanOrEqual(layout.viewportWidth);
  expect(layout.scrollX).toBe(0);
  await page.screenshot({
    path: testInfo.outputPath("strategy-workbench-390.png"),
    fullPage: true,
  });
});

test("boundary violation clears the previously rendered identity", async ({
  page,
}) => {
  let blocked = false;
  await page.unroute("**/api/mvp/v1/status");
  await page.route("**/api/mvp/v1/status", async (route) => {
    const payload = structuredClone(validStatusPayload);
    if (blocked) payload.boundaries.real_orders_submitted = true;
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(payload),
    });
  });
  await page.goto("overview");
  await expect(page.getByTestId("strategy-name")).toHaveText("btc-ema");
  blocked = true;
  await page.getByRole("button", { name: "刷新共享状态" }).click();
  await expect(page.getByText("策略工作台已阻断")).toBeVisible();
  await expect(page.getByTestId("strategy-name")).toHaveText("策略未加载");
});
