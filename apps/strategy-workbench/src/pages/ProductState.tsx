import { AlertTriangle, LoaderCircle, RefreshCw } from "lucide-react";

import { productErrorMessage } from "../features/product/presentation";
import styles from "./Pages.module.css";

export function ProductLoading({ label }: { label: string }) {
  return (
    <section className={styles.productState} aria-live="polite">
      <LoaderCircle className={styles.spin} aria-hidden="true" />
      <div>
        <strong>{label}</strong>
        <span>正在验证身份、来源与只读边界</span>
      </div>
    </section>
  );
}

export function ProductErrorState({
  error,
  onRetry,
  retrying = false,
}: {
  error: unknown;
  onRetry?: () => void | Promise<unknown>;
  retrying?: boolean;
}) {
  const message = productErrorMessage(error);
  return (
    <section
      className={`${styles.productState} ${styles.productStateError}`}
      role="alert"
    >
      <AlertTriangle aria-hidden="true" />
      <div>
        <strong>{message.title}</strong>
        <span>{message.detail}</span>
        {onRetry ? (
          <button
            type="button"
            onClick={() => void onRetry()}
            disabled={retrying}
          >
            <RefreshCw
              className={retrying ? styles.spin : undefined}
              aria-hidden="true"
            />
            {retrying ? "正在重试" : "重试明细"}
          </button>
        ) : null}
      </div>
    </section>
  );
}
