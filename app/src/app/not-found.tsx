import Link from "next/link";
import { EmptyState } from "@/components/armory/EmptyState";

export default function NotFound() {
  return (
    <div className="mx-auto max-w-2xl px-4 py-16 md:py-24" data-testid="not-found">
      <EmptyState k="d" as="h1" title="Page not found">
        <p>This page doesn&apos;t exist, or it has moved.</p>
        <p className="flex flex-wrap justify-center gap-3 pt-2 sm:justify-start">
          <Link href="/" className="btn btn-primary">Go home</Link>
          <Link href="/explore" className="btn">Explore launches</Link>
        </p>
      </EmptyState>
    </div>
  );
}
