import type { Dispatch } from "react";
import { openExportFolder } from "../api";
import { errorMessage } from "../errors";
import type { Action, State } from "../state";
import { S, nextSteps } from "../strings";
import { CheckCircle } from "./icons";

export function DoneScreen({ state, dispatch }: { state: State; dispatch: Dispatch<Action> }) {
  const target = state.target ?? state.source;
  const openFolder = () => {
    openExportFolder().catch((e: unknown) => dispatch({ type: "error", message: errorMessage(e) }));
  };
  return (
    <section className="screen done">
      <div className="titlebar" data-tauri-drag-region="deep" />
      <div className="form-page">
        <header className="hero">
          <CheckCircle className="done-mark" />
          <h1>{S.done}</h1>
        </header>
        <div className="group">
          <div className="row stacked">
            <span className="row-label">{S.savedTo}</span>
            <code className="mono path">{state.exportedPath}</code>
          </div>
        </div>
        <section>
          <h2 className="section-title">{S.nextStepsTitle}</h2>
          <ol className="group steps">
            {target !== null && nextSteps(target, state.format).map((step, n) => (
              <li key={step} className="row"><span className="step-number" aria-hidden="true">{n + 1}</span><span>{step}</span></li>
            ))}
          </ol>
          <p className="muted note section-footer">{S.repotFileNote}</p>
        </section>
        <div className="actions">
          <button className="large" onClick={() => dispatch({ type: "restart" })}>{S.startOver}</button>
          <button className="primary large" onClick={openFolder}>{S.openFolder}</button>
        </div>
      </div>
    </section>
  );
}
