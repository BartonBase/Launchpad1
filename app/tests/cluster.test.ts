import { describe, expect, it } from "vitest";
import { ClusterConfigError, MainnetForbiddenError, resolveCluster } from "@/config/cluster";

describe("resolveCluster", () => {
  it("defaults to localnet", () => {
    const c = resolveCluster({});
    expect(c.name).toBe("localnet");
    expect(c.label).toBe("LOCALNET");
    expect(c.rpcUrl).toBe("http://127.0.0.1:8899");
    expect(c.wsUrl).toBe("ws://127.0.0.1:8900");
  });

  it("treats empty string as default", () => {
    expect(resolveCluster({ cluster: "", rpcUrl: "" }).name).toBe("localnet");
  });

  it("supports devnet", () => {
    const c = resolveCluster({ cluster: "devnet" });
    expect(c.label).toBe("DEVNET");
    expect(c.rpcUrl).toBe("https://api.devnet.solana.com");
    expect(c.connectSrc).toContain("wss://api.devnet.solana.com");
  });

  it.each(["mainnet-beta", "mainnet", "MAINNET-BETA", " mainnet-beta "])(
    "rejects mainnet cluster name %j",
    (cluster) => {
      expect(() => resolveCluster({ cluster })).toThrow(MainnetForbiddenError);
    },
  );

  it.each([
    "https://api.mainnet-beta.solana.com",
    "https://API.MAINNET-BETA.SOLANA.COM/",
    "https://solana-mainnet.g.alchemy.com/v2/x",
    "https://mainnet.helius-rpc.com/?api-key=x",
  ])("rejects mainnet RPC url %s even when cluster says devnet", (rpcUrl) => {
    expect(() => resolveCluster({ cluster: "devnet", rpcUrl })).toThrow(MainnetForbiddenError);
    expect(() => resolveCluster({ rpcUrl })).toThrow(MainnetForbiddenError);
  });

  it("rejects unknown clusters instead of falling back", () => {
    expect(() => resolveCluster({ cluster: "testnet" })).toThrow(ClusterConfigError);
    expect(() => resolveCluster({ cluster: "custom" })).toThrow(ClusterConfigError);
  });

  it("allows RPC overrides only within the cluster's allowlist", () => {
    const c = resolveCluster({ cluster: "localnet", rpcUrl: "http://localhost:8899" });
    expect(c.rpcUrl).toBe("http://localhost:8899");
    expect(c.wsUrl).toBe("ws://localhost:8900");
    expect(() => resolveCluster({ cluster: "localnet", rpcUrl: "https://api.devnet.solana.com" })).toThrow(
      ClusterConfigError,
    );
    expect(() => resolveCluster({ cluster: "devnet", rpcUrl: "https://evil.example.com" })).toThrow(ClusterConfigError);
    expect(() => resolveCluster({ cluster: "devnet", rpcUrl: "https://u:p@api.devnet.solana.com" })).toThrow(
      ClusterConfigError,
    );
    expect(() => resolveCluster({ cluster: "devnet", rpcUrl: "not a url" })).toThrow(ClusterConfigError);
  });
});
