import {
  ProductApiContractError,
  ProductApiRequestError,
  ProductApiTransportError,
} from "../../api/productApi";
import type {
  RunEnvironment,
  RunLifecycle,
  RunResultStatus,
  RunRiskStatus,
} from "../../api/generated/productApi";

export const environmentLabels: Record<RunEnvironment, string> = {
  backtest: "Backtest",
  sandbox: "Demo",
  live: "Live",
};

export const lifecycleLabels: Record<RunLifecycle, string> = {
  created: "已创建",
  queued: "排队中",
  running: "运行中",
  stopping: "停止中",
  completed: "已完成",
  failed: "失败",
  cancelled: "已取消",
  stopped: "已停止",
};

export const resultLabels: Record<RunResultStatus, string> = {
  pending: "待生成",
  available: "可用",
  unavailable: "不可用",
};

export const riskLabels: Record<RunRiskStatus, string> = {
  pending: "待验证",
  active: "监控中",
  passed: "已通过",
  blocked: "已阻断",
};

export function formatTimestamp(value: number | null): string {
  if (value === null) return "未发生";
  return new Intl.DateTimeFormat("zh-CN", {
    dateStyle: "medium",
    timeStyle: "medium",
    hour12: false,
  }).format(new Date(value));
}

export function productErrorMessage(error: unknown): {
  title: string;
  detail: string;
} {
  if (error instanceof ProductApiContractError) {
    return {
      title: "产品合同验证失败",
      detail: `字段 ${error.field} 不符合只读产品合同，旧数据已清除。`,
    };
  }
  if (error instanceof ProductApiRequestError) {
    const title =
      error.status === 403
        ? "没有产品资源访问权限"
        : error.status === 404
          ? "产品资源不存在"
          : "产品服务返回错误";
    return {
      title,
      detail: `${error.message} · 请求 ${error.requestId}`,
    };
  }
  if (error instanceof ProductApiTransportError) {
    return {
      title: "产品服务不可用",
      detail: "浏览器未获得有效响应，请检查 Axum 服务状态。",
    };
  }
  return {
    title: "产品数据加载失败",
    detail: error instanceof Error ? error.message : "未知错误",
  };
}
