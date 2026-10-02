import Link from "next/link";
import { CLUSTER } from "@/config/cluster";
import { LAUNCH_TYPES, STATUS_LABEL, launchTypeStatus } from "@/config/armory";
import { TypeIcon } from "./TypeIcon";

/**
 * Launch types with their real deploy status. Per the sitemap, Tax split and Raffle appear wherever
 * types are listed, always as non-interactive "Coming soon" cards (FF_TAX_RAFFLE no longer hides them).
 */
export function LaunchTypes() {
  return (
    <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3" data-testid="launch-types">
      {LAUNCH_TYPES.map((t) => {
        const st = launchTypeStatus(t.id, CLUSTER.name);
        const soon = st === "coming-soon";
        const body = (
          <>
            <div className="flex items-center justify-between gap-2">
              <h4 className="flex items-center gap-2 font-semibold"><TypeIcon type={t.id} />{t.name}</h4>
              <span className={`tag ${st === "live" ? "tag-ok" : soon ? "tag-soon" : "tag-pd"}`}>{!soon && <i />}{STATUS_LABEL[st]}</span>
            </div>
            <p className="text-sm">{t.short}</p>
            <p className="text-muted text-sm">{t.description}</p>
          </>
        );
        return (
          <li key={t.id} data-testid={`launch-type-${t.id}`} data-status={st}>
            {soon ? (
              <div className="card card-soon h-full space-y-2 p-4" aria-disabled="true">{body}</div>
            ) : (
              <Link href={`/launch?type=${t.id}`} className="card block h-full space-y-2 p-4 hover:border-[var(--arm-color-border-strong)]">{body}</Link>
            )}
          </li>
        );
      })}
    </ul>
  );
}
