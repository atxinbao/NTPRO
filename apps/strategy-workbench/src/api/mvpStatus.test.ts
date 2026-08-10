import { describe, expect, it } from "vitest";

import { parseMvpStatus } from "./mvpStatus";
import { validStatusPayload } from "../test/fixtures";

describe("parseMvpStatus", () => {
  it("normalizes a valid sandbox status", () => {
    const result = parseMvpStatus(validStatusPayload);
    expect(result.strategyId).toBe("ema-cross");
    expect(result.environment).toBe("sandbox");
    expect(result.axes.tradingReadiness.status).toBe("blocked");
  });

  const invalidCases: Array<[string, (payload: Record<string, any>) => void]> =
    [
      [
        "missing strategy",
        (payload) => delete payload.identity.identities.strategy_id,
      ],
      [
        "identity mismatch",
        (payload) => {
          payload.status.identity_contract_id = "other";
        },
      ],
      [
        "identity formula mismatch",
        (payload) => {
          payload.identity.contract_id = "invalid";
          payload.status.identity_contract_id = "invalid";
        },
      ],
      [
        "consumer mismatch",
        (payload) => {
          payload.consumers = ["control_center"];
        },
      ],
      [
        "live environment",
        (payload) => {
          payload.identity.identities.environment = "live";
        },
      ],
      [
        "open root boundary",
        (payload) => {
          payload.boundaries.real_orders_submitted = true;
        },
      ],
      [
        "open identity boundary",
        (payload) => {
          payload.identity.boundaries.order_mutation_allowed = true;
        },
      ],
      [
        "readiness drift",
        (payload) => {
          payload.status.trading_readiness.status = "ready";
        },
      ],
      [
        "runtime status drift",
        (payload) => {
          payload.status.runtime.status = "ready";
        },
      ],
      [
        "missing source",
        (payload) => {
          payload.source_refs = [];
        },
      ],
    ];

  it.each(invalidCases)("fails closed for %s", (_name, mutate) => {
    const payload = structuredClone(validStatusPayload) as Record<string, any>;
    mutate(payload);
    expect(() => parseMvpStatus(payload)).toThrow("共享状态合同无效");
  });
});
