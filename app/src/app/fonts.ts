/**
 * Self-hosted fonts. next/font downloads them at BUILD time and serves them from our own
 * origin, so the CSP stays closed to Google Fonts at runtime (font-src 'self').
 */
import { Bricolage_Grotesque, Geist, Geist_Mono } from "next/font/google";

export const geist = Geist({ subsets: ["latin"], variable: "--font-geist", display: "swap" });
export const geistMono = Geist_Mono({ subsets: ["latin"], variable: "--font-geist-mono", display: "swap" });
// Headlines + the text wordmark only (ExtraBold, condensed via font-stretch 82% / opsz 96).
export const bricolage = Bricolage_Grotesque({
  subsets: ["latin"],
  variable: "--font-bricolage",
  display: "swap",
  axes: ["opsz", "wdth"],
});

export const fontVariables = `${geist.variable} ${geistMono.variable} ${bricolage.variable}`;
