import type { State } from "../state";
import { S, categoryLabel, statusText } from "../strings";

// Deliberately has no buttons: the user cannot leave until the scan finishes.
export function ScanScreen({ state }: { state: State }) {
  return (
    <section className="screen scan">
      <h1>{S.scanning}</h1>
      <p className="muted">{S.scanIntro}</p>
      <ul className="card providers" aria-live="polite">
        {state.providers.map((p) => {
          const kind = typeof p.status === "string" ? p.status : Object.keys(p.status)[0];
          return (
            <li key={p.id} className={`provider status-${kind}`}>
              <span className={kind === "running" ? "spinner" : "dot"} aria-hidden="true" />
              <span className="provider-id">{p.id}</span>
              <span className="muted">{categoryLabel(p.category)}</span>
              <span className="provider-status">{statusText(p.status)}</span>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
