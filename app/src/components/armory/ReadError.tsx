export function ReadError({ what, error }: { what: string; error: string }) {
  return (
    <div role="status" className="card border-warning-border bg-warning-bg text-warning p-4 text-sm" data-testid="read-error">
      Couldn&apos;t load {what} from the RPC right now. The public devnet endpoint rate-limits; refresh in a moment.
      <span className="text-dim mt-1 block font-mono text-xs break-all">{error.slice(0, 160)}</span>
    </div>
  );
}
