import { type Dispatch, useEffect, useMemo, useState } from 'react';
import { chooseFolder, exportTo, previewExport } from '../api';
import { errorMessage } from '../errors';
import { buildSelection, currentTarget, fileTree, warningItemId } from '../logic';
import type { Action, State } from '../state';
import { S, warningText, withCount } from '../strings';
import type { ExportBundle, Format, OutputFile } from '../types';
import { DocIcon, TerminalIcon, WarningTriangle } from './icons';

export function ExportScreen({ state, dispatch }: { state: State; dispatch: Dispatch<Action> }) {
  const [bundle, setBundle] = useState<ExportBundle | null>(null);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [previewFailed, setPreviewFailed] = useState(false);
  const selection = useMemo(
    () => buildSelection(state.checked, state.revealed),
    [state.checked, state.revealed],
  );
  // The same Target the review screen judged with, so the preview and the saved files match it.
  const target = useMemo(
    () => currentTarget({ target: state.target, packages: state.packages }),
    [state.target, state.packages],
  );
  const { format } = state;

  useEffect(() => {
    if (target === null) return;
    let current = true;
    setBundle(null);
    setPreviewFailed(false);
    previewExport(selection, target, format)
      .then((next) => {
        if (!current) return;
        setBundle(next);
        setSelectedPath((path) =>
          next.files.some((f) => f.path === path) ? path : (next.files[0]?.path ?? null),
        );
      })
      .catch((e: unknown) => {
        if (!current) return;
        setPreviewFailed(true);
        dispatch({ type: 'error', message: errorMessage(e) });
      });
    return () => {
      current = false;
    };
  }, [selection, target, format, dispatch]);

  const onSave = async () => {
    if (target === null) return;
    try {
      const parent = await chooseFolder();
      if (parent === null) return;
      setSaving(true);
      const path = await exportTo(selection, target, format, parent);
      dispatch({ type: 'exported', path });
    } catch (e) {
      // Stay on this screen so the user can pick another folder.
      dispatch({ type: 'error', message: errorMessage(e) });
    } finally {
      setSaving(false);
    }
  };

  return (
    <ExportView
      state={state}
      dispatch={dispatch}
      bundle={bundle}
      previewFailed={previewFailed}
      selectedPath={selectedPath}
      onSelect={setSelectedPath}
      onSave={onSave}
      saving={saving}
    />
  );
}

interface ViewProps {
  state: State;
  dispatch: Dispatch<Action>;
  bundle: ExportBundle | null;
  previewFailed?: boolean;
  selectedPath: string | null;
  onSelect: (path: string) => void;
  onSave: () => void;
  saving: boolean;
}

export function ExportView({
  state,
  dispatch,
  bundle,
  previewFailed = false,
  selectedPath,
  onSelect,
  onSave,
  saving,
}: ViewProps) {
  const names = new Map((state.scan?.items ?? []).map((i) => [i.id, i.name]));
  const tree = bundle === null ? null : fileTree(bundle.files);
  const selected = bundle?.files.find((f) => f.path === selectedPath) ?? null;
  const fileButton = (f: OutputFile, label: string) => (
    <li key={f.path}>
      <button
        type="button"
        className={`file ${f.path === selectedPath ? 'active' : ''}`}
        aria-current={f.path === selectedPath ? 'true' : undefined}
        onClick={() => onSelect(f.path)}
      >
        {f.executable ? <TerminalIcon /> : <DocIcon />}
        <span className="file-name">{label}</span>
      </button>
    </li>
  );
  const formats: [Format, string][] = [
    ['script', S.formatScript],
    ['manifest', S.formatManifest],
  ];

  return (
    <section className="screen export">
      <header className="toolbar" data-tauri-drag-region="deep">
        <div className="toolbar-title">
          <h1>{S.exportTitle}</h1>
        </div>
        <div className="spacer" />
        <fieldset className="segmented" disabled={saving}>
          <legend className="visually-hidden">{S.format}</legend>
          {formats.map(([value, label]) => (
            <label key={value}>
              <input
                type="radio"
                name="format"
                value={value}
                checked={state.format === value}
                onChange={() => dispatch({ type: 'setFormat', format: value })}
              />
              <span>{label}</span>
            </label>
          ))}
        </fieldset>
        <div className="spacer" />
        <button type="button" disabled={saving} onClick={() => dispatch({ type: 'backToReview' })}>
          {S.back}
        </button>
        <button
          type="button"
          className="primary"
          disabled={saving || bundle === null}
          onClick={onSave}
        >
          {saving ? S.saving : S.save}
        </button>
      </header>
      {bundle !== null && bundle.warnings.length > 0 && (
        <div className="warnings" role="status">
          <WarningTriangle className="symbol secret" />
          <div>
            <h2>{S.warningsTitle}</h2>
            <ul>
              {bundle.warnings.map((w, n) => {
                const id = warningItemId(w);
                return <li key={n}>{warningText(w, names.get(id) ?? id)}</li>;
              })}
            </ul>
          </div>
        </div>
      )}
      {tree === null ? (
        <p className="muted preview-status">{previewFailed ? S.previewFailed : S.previewLoading}</p>
      ) : (
        <div className="preview">
          <nav className="tree">
            {bundle?.files.length === 0 && <p className="muted">{S.previewEmpty}</p>}
            <ul>
              {tree.files.map((f) => fileButton(f, f.path))}
              {tree.folders.map((folder) => (
                <li key={folder.name}>
                  <details>
                    <summary>{withCount(`${folder.name}/`, folder.files.length)}</summary>
                    <ul>
                      {folder.files.map((f) => fileButton(f, f.path.slice(folder.name.length + 1)))}
                    </ul>
                  </details>
                </li>
              ))}
            </ul>
          </nav>
          <div className="contents">
            {selected !== null && (
              <>
                <div className="contents-header">
                  <span className="mono">{selected.path}</span>
                </div>
                <pre className="code">
                  {selected.contents.split('\n').map((line, n) => (
                    <span key={n} className="line">
                      {line}
                    </span>
                  ))}
                </pre>
              </>
            )}
          </div>
        </div>
      )}
    </section>
  );
}
