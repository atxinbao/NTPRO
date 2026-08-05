import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { router } from "./router";

describe("strategy workbench foundation", () => {
  it("renders verified status and keeps unreleased product sections disabled", async () => {
    window.history.replaceState({}, "", "/strategy-workbench/overview");
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );

    expect(await screen.findByText("策略状态已验证")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-name")).toHaveTextContent("btc-ema");
    expect(
      screen
        .getAllByRole("button", { name: /Live/ })
        .every((button) => button.hasAttribute("disabled")),
    ).toBe(true);
    expect(
      screen.queryByRole("button", { name: /下单|撤单|改单|平仓/ }),
    ).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole("link", { name: "系统状态" }));
    expect(
      await screen.findByRole("heading", { name: "系统状态" }),
    ).toBeInTheDocument();
  });
});
