import { createClient } from "./generated/productApi/client";
import {
  getRun,
  getStrategy,
  getStrategyVersion,
  listRuns,
  listStrategies,
  listStrategyVersions,
  type GetRunData,
  type GetStrategyData,
  type GetStrategyVersionData,
  type ListRunsData,
  type ListStrategiesData,
  type ListStrategyVersionsData,
  type ProductErrorResponse,
  type RunDetailResponse,
  type RunListResponse,
  type StrategyDetailResponse,
  type StrategyListResponse,
  type StrategyVersionDetailResponse,
  type StrategyVersionListResponse,
} from "./generated/productApi";
import {
  zProductErrorResponse,
  zRunDetailResponse,
  zRunListResponse,
  zStrategyDetailResponse,
  zStrategyListResponse,
  zStrategyVersionDetailResponse,
  zStrategyVersionListResponse,
} from "./generated/productApi/zod.gen";
import type { z } from "zod";

const PRODUCT_API_BASE_URL = "/api/product/v1";

type ListStrategiesQuery = NonNullable<ListStrategiesData["query"]>;
type StrategyPath = GetStrategyData["path"];
type ListStrategyVersionsQuery = NonNullable<ListStrategyVersionsData["query"]>;
type StrategyVersionPath = GetStrategyVersionData["path"];
type ListRunsQuery = NonNullable<ListRunsData["query"]>;
type RunPath = GetRunData["path"];

interface ProductApiClientOptions {
  baseUrl?: string;
  fetch?: typeof fetch;
}

interface RequestFields {
  data: unknown;
  error: unknown;
  response?: Response;
}

function resolveBaseUrl(baseUrl: string): string {
  if (/^https?:\/\//u.test(baseUrl)) return baseUrl;
  if (typeof globalThis.location === "undefined") return baseUrl;
  return new URL(baseUrl, globalThis.location.origin)
    .toString()
    .replace(/\/$/u, "");
}

export class ProductApiContractError extends Error {
  readonly field: string;

  constructor(field: string, cause?: unknown) {
    super(`产品 API 合同无效：${field}`, { cause });
    this.name = "ProductApiContractError";
    this.field = field;
  }
}

export class ProductApiRequestError extends Error {
  readonly status: number;
  readonly requestId: string;
  readonly code: ProductErrorResponse["error"]["code"];
  readonly field: string;
  readonly retryable: boolean;

  constructor(status: number, response: ProductErrorResponse) {
    super(response.error.summary);
    this.name = "ProductApiRequestError";
    this.status = status;
    this.requestId = response.request_id;
    this.code = response.error.code;
    this.field = response.error.field;
    this.retryable = response.error.retryable;
  }
}

export class ProductApiTransportError extends Error {
  constructor(cause: unknown) {
    super("产品 API 请求未获得有效 HTTP 响应", { cause });
    this.name = "ProductApiTransportError";
  }
}

function canonicalJson(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(canonicalJson);
  if (typeof value !== "object" || value === null) return value;
  return Object.fromEntries(
    Object.entries(value)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, entry]) => [key, canonicalJson(entry)]),
  );
}

function parseStrict<T>(
  schema: z.ZodType<T>,
  payload: unknown,
  field: string,
): T {
  const result = schema.safeParse(payload);
  if (!result.success) throw new ProductApiContractError(field, result.error);
  if (
    JSON.stringify(canonicalJson(result.data)) !==
    JSON.stringify(canonicalJson(payload))
  ) {
    throw new ProductApiContractError(`${field}.unknown_or_defaulted_field`);
  }
  return result.data;
}

function assertPage(
  payload: StrategyListResponse | StrategyVersionListResponse | RunListResponse,
  field: string,
): void {
  if (payload.page.returned_count !== payload.data.length) {
    throw new ProductApiContractError(`${field}.page.returned_count`);
  }
  if (payload.page.returned_count > payload.page.limit) {
    throw new ProductApiContractError(`${field}.page.limit`);
  }
  const hasCursor = payload.page.next_cursor !== null;
  if (payload.page.has_more !== hasCursor) {
    throw new ProductApiContractError(`${field}.page.next_cursor`);
  }
  if (hasCursor && payload.page.next_cursor?.trim() === "") {
    throw new ProductApiContractError(`${field}.page.next_cursor`);
  }
  if (payload.data.length === 0 && payload.page.has_more) {
    throw new ProductApiContractError(`${field}.page.has_more`);
  }
}

async function resolveResponse<T>(
  request: Promise<RequestFields>,
  schema: z.ZodType<T>,
  field: string,
  paginated = false,
): Promise<T> {
  let result: RequestFields;
  try {
    result = await request;
  } catch (error) {
    throw new ProductApiTransportError(error);
  }

  if (result.data !== undefined) {
    const parsed = parseStrict(schema, result.data, field);
    if (paginated) {
      assertPage(
        parsed as
          StrategyListResponse | StrategyVersionListResponse | RunListResponse,
        field,
      );
    }
    return parsed;
  }

  if (!result.response) throw new ProductApiTransportError(result.error);
  const error = parseStrict(
    zProductErrorResponse,
    result.error,
    `${field}.error`,
  );
  throw new ProductApiRequestError(result.response.status, error);
}

export function createProductApiClient(options: ProductApiClientOptions = {}) {
  const client = createClient({
    baseUrl: resolveBaseUrl(options.baseUrl ?? PRODUCT_API_BASE_URL),
    credentials: "same-origin",
    fetch: options.fetch,
    headers: { Accept: "application/json" },
  });

  return {
    listStrategies(
      query?: ListStrategiesQuery,
      signal?: AbortSignal,
    ): Promise<StrategyListResponse> {
      return resolveResponse(
        listStrategies({ client, query, signal }),
        zStrategyListResponse,
        "strategy_list",
        true,
      );
    },

    getStrategy(
      path: StrategyPath,
      signal?: AbortSignal,
    ): Promise<StrategyDetailResponse> {
      return resolveResponse(
        getStrategy({ client, path, signal }),
        zStrategyDetailResponse,
        "strategy_detail",
      );
    },

    listStrategyVersions(
      path: ListStrategyVersionsData["path"],
      query?: ListStrategyVersionsQuery,
      signal?: AbortSignal,
    ): Promise<StrategyVersionListResponse> {
      return resolveResponse(
        listStrategyVersions({ client, path, query, signal }),
        zStrategyVersionListResponse,
        "strategy_version_list",
        true,
      );
    },

    getStrategyVersion(
      path: StrategyVersionPath,
      signal?: AbortSignal,
    ): Promise<StrategyVersionDetailResponse> {
      return resolveResponse(
        getStrategyVersion({ client, path, signal }),
        zStrategyVersionDetailResponse,
        "strategy_version_detail",
      );
    },

    listRuns(
      query?: ListRunsQuery,
      signal?: AbortSignal,
    ): Promise<RunListResponse> {
      return resolveResponse(
        listRuns({ client, query, signal }),
        zRunListResponse,
        "run_list",
        true,
      );
    },

    getRun(path: RunPath, signal?: AbortSignal): Promise<RunDetailResponse> {
      return resolveResponse(
        getRun({ client, path, signal }),
        zRunDetailResponse,
        "run_detail",
      );
    },
  };
}

export const productApi = createProductApiClient();
