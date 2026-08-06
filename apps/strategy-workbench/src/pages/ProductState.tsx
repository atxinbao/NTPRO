import { AlertTriangle, LoaderCircle } from "lucide-react";

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

export function ProductErrorState({ error }: { error: unknown }) {
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
      </div>
    </section>
  );
}
