import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

export default defineConfig({
  resolve: {
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  test: {
    environment: "node",
    include: ["tests/**/*.test.ts"],
    // Never inherit a developer's real env: tests set cluster env explicitly.
    env: { NEXT_PUBLIC_SOLANA_CLUSTER: "localnet", NEXT_PUBLIC_SOLANA_RPC_URL: "" },
  },
});
