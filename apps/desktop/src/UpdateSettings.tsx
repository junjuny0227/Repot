import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { format, messages, useLocale } from "./i18n";
import { updates } from "./updateService";
import type { Screen } from "./state";
import type { UpdateState } from "./updater";

function Status({ state }: { state: UpdateState }) {
  const t = messages().updates;
  if (state.phase === "error") return <p role="alert">{t.failed} <span>{state.error}</span></p>;
  const label = state.phase === "checking" ? t.checking : state.phase === "current" ? t.current : state.phase === "unsupported" ? t.unsupported : "";
  return label ? <p role="status">{label}</p> : null;
}

/** A native dialog keeps keyboard focus inside the sheet and makes the scan inert. */
export function UpdateNotice({ screen }: { screen: Screen }) {
  useLocale();
  const state = useSyncExternalStore(updates.subscribe, updates.getSnapshot, updates.getSnapshot);
  updates.setScreen(screen);
  const ref = useRef<HTMLDialogElement>(null);
  const offer = updates.shouldOffer();
  const t = messages().updates;
  useEffect(() => {
    const dialog = ref.current;
    if (offer && dialog && !dialog.open) dialog.showModal();
    if (!offer && dialog?.open) dialog.close();
  }, [offer]);
  return <dialog ref={ref} className="update-sheet" aria-labelledby="update-title" onCancel={(event) => {
    event.preventDefault();
    updates.later();
  }}>
    <h2 id="update-title">{format(t.available, { version: state.version })}</h2>
    <p>{t.intro}</p>
    {state.notes && <pre className="update-notes">{state.notes}</pre>}
    {updates.busy() ? <div className="update-progress" role="status">
      <p>{state.phase === "restarting" ? t.restarting : t.downloading}</p>
      <progress aria-label={t.downloading} value={state.total ? Math.min(state.downloaded, state.total) : undefined} max={state.total || 1} />
      <p>{format(t.progress, { downloaded: state.downloaded, total: state.total ?? "—" })}</p>
    </div> : <div className="actions">
      <button onClick={() => updates.later()}>{t.later}</button>
      <button className="primary" onClick={() => void updates.install()}>{t.install}</button>
    </div>}
  </dialog>;
}

export function UpdateSettings() {
  useLocale();
  const [open, setOpen] = useState(false);
  const state = useSyncExternalStore(updates.subscribe, updates.getSnapshot, updates.getSnapshot);
  const ref = useRef<HTMLDialogElement>(null);
  const t = messages().updates;
  useEffect(() => {
    if (open && !ref.current?.open) ref.current?.showModal();
    if (!open && ref.current?.open) ref.current.close();
  }, [open]);
  return <>
    <button className="settings-button" onClick={() => setOpen(true)}>{t.settings}</button>
    <dialog ref={ref} className="update-sheet settings-sheet" aria-labelledby="settings-title" onCancel={() => setOpen(false)}>
      <h2 id="settings-title">{t.settings}</h2>
      <label className="update-toggle"><input type="checkbox" checked={state.automatic} onChange={(event) => updates.setAutomatic(event.target.checked)} />{t.automatic}</label>
      <p className="muted">{t.privacy}</p>
      <Status state={state} />
      {state.phase === "available" && <p role="status">{format(t.deferred, { version: state.version })}</p>}
      <div className="actions">
        <button disabled={state.phase === "checking" || updates.busy()} onClick={() => { setOpen(false); void updates.check(); }}>{t.checkNow}</button>
        <button className="primary" onClick={() => setOpen(false)}>{t.done}</button>
      </div>
    </dialog>
  </>;
}

/** Non-modal status remains available on safe screens after a failed installation. */
export function UpdateStatus({ screen }: { screen: Screen }) {
  useLocale();
  const state = useSyncExternalStore(updates.subscribe, updates.getSnapshot, updates.getSnapshot);
  if (screen !== "start" && screen !== "done") return null;
  return <div className="update-status"><Status state={state} /></div>;
}
