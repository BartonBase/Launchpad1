import { describe, expect, it } from "vitest";
import { ClusterConfigError, explorerAddressUrl, explorerTxUrl, resolveCluster, serverRpcUrl } from "@/config/cluster";

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

  it.each(["mainnet-beta", "mainnet", "MAINNET-BETA", " mainnet-beta "])("accepts mainnet cluster name %j (explicit opt-in)", (cluster) => {
    const c = resolveCluster({ cluster });
    expect(c.name).toBe("mainnet-beta");
    expect(c.isMainnet).toBe(true);
    expect(c.displayName).toBe("Mainnet");
    expect(c.rpcUrl).toBe("https://api.mainnet-beta.solana.com");
    expect(c.publicRpc).toBe(true);
    expect(c.connectSrc).toEqual(["https://api.mainnet-beta.solana.com", "wss://api.mainnet-beta.solana.com"]);
  });

  it("accepts a Helius mainnet RPC on mainnet, keeping the api-key query and deriving wss", () => {
    const c = resolveCluster({ cluster: "mainnet-beta", rpcUrl: "https://mainnet.helius-rpc.com/?api-key=abc123" });
    expect(c.rpcUrl).toBe("https://mainnet.helius-rpc.com/?api-key=abc123");
    expect(c.wsUrl).toBe("wss://mainnet.helius-rpc.com/?api-key=abc123");
    expect(c.publicRpc).toBe(false);
    expect(c.connectSrc).toEqual(["https://mainnet.helius-rpc.com", "wss://mainnet.helius-rpc.com"]);
  });

  it.each([
    "https://api.mainnet-beta.solana.com",
    "https://API.MAINNET-BETA.SOLANA.COM/",
    "https://solana-mainnet.g.alchemy.com/v2/x",
    "https://mainnet.helius-rpc.com/?api-key=x",
  ])("rejects mainnet RPC url %s on a devnet build", (rpcUrl) => {
    expect(() => resolveCluster({ cluster: "devnet", rpcUrl })).toThrow(ClusterConfigError);
    expect(() => resolveCluster({ rpcUrl })).toThrow(ClusterConfigError); // localnet default
  });

  it.each(["https://api.devnet.solana.com", "https://devnet.helius-rpc.com/?api-key=x", "https://api.testnet.solana.com", "https://localhost:8899", "http://mainnet.helius-rpc.com/"])(
    "rejects %s on a mainnet build",
    (rpcUrl) => {
      expect(() => resolveCluster({ cluster: "mainnet-beta", rpcUrl })).toThrow(ClusterConfigError);
    },
  );

  it("explorer links carry ?cluster= except on mainnet", () => {
    const dev = resolveCluster({ cluster: "devnet" });
    const main = resolveCluster({ cluster: "mainnet-beta" });
    expect(explorerTxUrl("sig", dev)).toBe("https://explorer.solana.com/tx/sig?cluster=devnet");
    expect(explorerTxUrl("sig", main)).toBe("https://explorer.solana.com/tx/sig");
    expect(explorerAddressUrl("Addr", main)).toBe("https://explorer.solana.com/address/Addr");
  });

  it("serverRpcUrl uses the server-only SOLANA_RPC_URL when set, validated for the cluster", () => {
    const main = resolveCluster({ cluster: "mainnet-beta" });
    expect(serverRpcUrl(main, undefined)).toBe("https://api.mainnet-beta.solana.com");
    expect(serverRpcUrl(main, " ")).toBe("https://api.mainnet-beta.solana.com");
    expect(serverRpcUrl(main, "https://mainnet.helius-rpc.com/?api-key=srv")).toBe("https://mainnet.helius-rpc.com/?api-key=srv");
    expect(() => serverRpcUrl(main, "https://api.devnet.solana.com")).toThrow(ClusterConfigError);
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
    // Custom https RPCs (Helius, Triton, ...) are allowed on devnet/mainnet; http and credentials are not.
    expect(resolveCluster({ cluster: "devnet", rpcUrl: "https://rpc.example.com" }).rpcUrl).toBe("https://rpc.example.com");
    expect(() => resolveCluster({ cluster: "devnet", rpcUrl: "http://rpc.example.com" })).toThrow(ClusterConfigError);
    expect(() => resolveCluster({ cluster: "devnet", rpcUrl: "https://u:p@api.devnet.solana.com" })).toThrow(
      ClusterConfigError,
    );
    expect(() => resolveCluster({ cluster: "devnet", rpcUrl: "not a url" })).toThrow(ClusterConfigError);
  });
});
