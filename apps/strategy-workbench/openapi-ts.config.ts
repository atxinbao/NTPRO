import { defineConfig } from "@hey-api/openapi-ts";

export default defineConfig({
  input: "../../docs/product/api/ntpro_product_v1.openapi.json",
  output: "src/api/generated/productApi",
  plugins: [
    "@hey-api/client-fetch",
    "@hey-api/typescript",
    "@hey-api/sdk",
    { name: "zod", compatibilityVersion: 4 },
  ],
});
