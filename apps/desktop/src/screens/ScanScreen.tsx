import type { State } from '../state';
import { categoryLabel, S, statusText } from '../strings';
import type { ProviderStatus } from '../types';
import { CheckCircle, ClockCircle, ExclamationCircle, MinusCircle, Spinner } from './icons';

function StatusIcon({ status }: { status: ProviderStatus }) {
  if (status === 'running') return <Spinner />;
  if (status === 'notDetected') return <MinusCircle className="symbol status-icon" />;
  if (status === 'timedOut') return <ClockCircle className="symbol status-icon" />;
  if ('ok' in status) return <CheckCircle className="symbol status-icon" />;
  return <ExclamationCircle className="symbol status-icon" />;
}

// Deliberately has no buttons: the user cannot leave until the scan finishes.
export function ScanScreen({ state }: { state: State }) {
  return (
    <section className="screen scan">
      <div className="titlebar" data-tauri-drag-region="deep" />
      <div className="form-page wide">
        <header className="page-header">
          <Spinner large />
          <div>
            <h1>{S.scanning}</h1>
            <p className="muted">{S.scanIntro}</p>
          </div>
        </header>
        <ul className="group providers" aria-live="polite">
          {state.providers.map((p) => {
            const kind = typeof p.status === 'string' ? p.status : Object.keys(p.status)[0];
            return (
              <li key={p.id} className={`row provider status-${kind}`}>
                <StatusIcon status={p.status} />
                <span className="provider-id">{p.id}</span>
                <span className="provider-category">{categoryLabel(p.category)}</span>
                <span className="provider-status">{statusText(p.status)}</span>
              </li>
            );
          })}
        </ul>
      </div>
    </section>
  );
}
