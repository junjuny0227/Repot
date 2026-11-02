import { useEffect, useMemo, useState, type Dispatch } from "react";
import { chooseFolder, exportTo, previewExport } from "../api";
import { errorMessage } from "../errors";
import { buildSelection, fileTree, warningItemId } from "../logic";
import type { Action, State } from "../state";
import { S, warningText, withCount } from "../strings";
import type { ExportBundle, Format, OutputFile } from "../types";

export function ExportScreen({ state, dispatch }: { state: State; dispatch: Dispatch<Action> }) {
  const [bundle, setBundle] = useState<ExportBundle | null>(null);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const selection = useMemo(() => buildSelection(state.checked, state.revealed), [state.checked, state.revealed]);
  const { target, format } = state;

  useEffect(() => {
    if (target === null) return;
    let current = true;
    setBundle(null);
    previewExport(selection, target, format)
      .then((next) => {
        if (!current) return;
        setBundle(next);
        setSelectedPath((path) => (next.files.some((f) => f.path === path) ? path : next.files[0]?.path ?? null));
      })
      .catch((e: unknown) => { if (current) dispatch({ type: "error", message: errorMessage(e) }); });
    return () => { current = false; };
  }, [selection, target, format, dispatch]);

  const onSave = async () => {
    if (target === null) return;
    try {
      const parent = await chooseFolder();
      if (parent === null) return;
      setSaving(true);
      const path = await exportTo(selection, target, format, parent);
      dispatch({ type: "exported", path });
    } catch (e) {
      // Stay on this screen so the user can pick another folder.
      dispatch({ type: "error", message: errorMessage(e) });
    } finally {
      setSaving(false);
    }
  };

  return <ExportView state={state} dispatch={dispatch} bundle={bundle} selectedPath={selectedPath} onSelect={setSelectedPath} onSave={onSave} saving={saving} />;
}

interface ViewProps {
  state: State;
  dispatch: Dispatch<Action>;
  bundle: ExportBundle | null;
  selectedPath: string | null;
  onSelect: (path: string) => void;
  onSave: () => void;
  saving: boolean;
}

export function ExportView({ state, dispatch, bundle, selectedPath, onSelect, onSave, saving }: ViewProps) {
  const names = new Map((state.scan?.items ?? []).map((i) => [i.id, i.name]));
  const tree = bundle === null ? null : fileTree(bundle.files);
  const selected = bundle?.files.find((f) => f.path === selectedPath) ?? null;
  const fileButton = (f: OutputFile, label: string) => (
    <li key={f.path}>
      <button className={`file ${f.path === selectedPath ? "active" : ""}`} onClick={() => onSelect(f.path)}>{label}</button>
    </li>
  );
  const formats: [Format, string][] = [["script", S.formatScript], ["manifest", S.formatManifest]];

  return (
    <section className="screen export">
      <header className="toolbar">
        <h1>{S.exportTitle}</h1>
        <fieldset className="formats" disabled={saving}>
          <legend>{S.format}</legend>
          {formats.map(([value, label]) => (
            <label key={value}>
              <input type="radio" name="format" value={value} checked={state.format === value} onChange={() => dispatch({ type: "setFormat", format: value })} />
              {label}
            </label>
          ))}
        </fieldset>
        <button disabled={saving} onClick={() => dispatch({ type: "backToReview" })}>{S.back}</button>
        <button className="primary" disabled={saving || bundle === null} onClick={onSave}>{saving ? S.saving : S.save}</button>
      </header>
      {bundle !== null && bundle.warnings.length > 0 && (
        <div className="warnings card">
          <h3>{S.warningsTitle}</h3>
          <ul>
            {bundle.warnings.map((w, n) => {
              const id = warningItemId(w);
              return <li key={n}>{warningText(w, names.get(id) ?? id)}</li>;
            })}
          </ul>
        </div>
      )}
      {tree === null ? (
        <p className="muted">{S.previewLoading}</p>
      ) : (
        <div className="preview">
          <nav className="tree card">
            {bundle?.files.length === 0 && <p className="muted">{S.previewEmpty}</p>}
            <ul>
              {tree.files.map((f) => fileButton(f, f.path))}
              {tree.folders.map((folder) => (
                <li key={folder.name}>
                  <details><summary>{withCount(`${folder.name}/`, folder.files.length)}</summary>
                    <ul>{folder.files.map((f) => fileButton(f, f.path.slice(folder.name.length + 1)))}</ul>
                  </details>
                </li>
              ))}
            </ul>
          </nav>
          <div className="contents card">
            {selected !== null && <pre>{selected.contents}</pre>}
          </div>
        </div>
      )}
    </section>
  );
}
