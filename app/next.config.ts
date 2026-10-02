import type { NextConfig } from "next";
// Importing the cluster config here makes `next dev` / `next build` fail fast
// (MainnetForbiddenError) if a mainnet cluster or RPC URL is configured.
import { CLUSTER } from "./src/config/cluster";

const isDev = process.env.NODE_ENV !== "production";

/**
 * Content-Security-Policy.
 * - connect-src: only the ACTIVE cluster's RPC/WS origins (localnet:
 *   127.0.0.1|localhost:8899/8900, devnet: api.devnet.solana.com https/wss).
 * - script-src: Next.js App Router emits inline bootstrap/RSC scripts, so
 *   without per-request nonces 'unsafe-inline' is required. 'unsafe-eval' is
 *   dev-only (React debugging). See README "CSP" for the nonce upgrade path.
 * - devnet only: Switchboard oracle gateways (https://<ipv4>.xip.switchboard-oracles.xyz) for
 *   the randomness reveal. A host wildcard, because oracles can rotate; the app additionally
 *   checks every gateway URL against SWITCHBOARD_GATEWAY_RE before calling it.
 * - No other third-party origins (no fonts, analytics, CDNs).
 */
const connectSrc = [
  "'self'",
  ...CLUSTER.connectSrc,
  ...(CLUSTER.name === "devnet" ? ["https://*.xip.switchboard-oracles.xyz"] : []),
  ...(isDev ? ["ws://localhost:3000", "ws://127.0.0.1:3000"] : []),
];

const csp = [
  "default-src 'self'",
  `script-src 'self' 'unsafe-inline'${isDev ? " 'unsafe-eval'" : ""}`,
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: blob:",
  "font-src 'self'",
  `connect-src ${connectSrc.join(" ")}`,
  "frame-src 'none'",
  "worker-src 'self' blob:",
  "manifest-src 'self'",
  "media-src 'self'",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'self'",
  "frame-ancestors 'none'",
  // Only upgrade when every allowed endpoint is https/wss (devnet prod build);
  // on localnet it would break http://127.0.0.1:8899.
  ...(!isDev && CLUSTER.name === "devnet" ? ["upgrade-insecure-requests"] : []),
].join("; ");

const securityHeaders = [
  { key: "Content-Security-Policy", value: csp },
  { key: "X-Frame-Options", value: "DENY" },
  { key: "X-Content-Type-Options", value: "nosniff" },
  { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
  {
    key: "Permissions-Policy",
    value:
      "camera=(), microphone=(), geolocation=(), payment=(), usb=(), serial=(), hid=(), " +
      "midi=(), magnetometer=(), gyroscope=(), accelerometer=(), display-capture=(), browsing-topics=()",
  },
  { key: "Cross-Origin-Opener-Policy", value: "same-origin-allow-popups" },
  { key: "Cross-Origin-Resource-Policy", value: "same-origin" },
  { key: "X-DNS-Prefetch-Control", value: "off" },
  ...(!isDev ? [{ key: "Strict-Transport-Security", value: "max-age=63072000; includeSubDomains" }] : []),
];

const nextConfig: NextConfig = {
  reactStrictMode: true,
  poweredByHeader: false,
  async redirects() {
    return [
      // Pre-Armory routes.
      { source: "/collections/:mint", destination: "/t/:mint", permanent: false },
      { source: "/token/:mint", destination: "/t/:mint", permanent: false },
      { source: "/lottery", destination: "/explore", permanent: false },
    ];
  },
  async headers() {
    return [{ source: "/:path*", headers: securityHeaders }];
  },
};

export default nextConfig;
