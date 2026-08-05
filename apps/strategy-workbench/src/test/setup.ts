import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterAll, afterEach, beforeAll } from "vitest";

import { server } from "./server";

beforeAll(() => server.listen({ onUnhandledRequest: "error" }));
afterEach(() => {
  cleanup();
  server.resetHandlers();
});
afterAll(() => server.close());

Object.defineProperty(window, "matchMedia", {
  configurable: true,
  value: () => ({
    matches: true,
    addEventListener() {},
    removeEventListener() {},
  }),
});

Object.defineProperty(window, "scrollTo", {
  configurable: true,
  value: () => undefined,
});
