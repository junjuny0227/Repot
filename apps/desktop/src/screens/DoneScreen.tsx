import type { Dispatch } from "react";
import { openExportFolder } from "../api";
import { errorMessage } from "../errors";
import type { Action, State } from "../state";
import { S, nextSteps } from "../strings";

export function DoneScreen({ state, dispatch }: { state: State; dispatch: Dispatch<Action> }) {
  const target = state.target ?? state.source;
  const openFolder = () => {
    openExportFolder().catch((e: unknown) => dispatch({ type: "error", message: errorMessage(e) }));
  };
  return (
    <section className="screen done">
      <h1>{S.done}</h1>
      <div className="card form">
        <div className="field">
          <span>{S.savedTo}</span>
          <code className="mono">{state.exportedPath}</code>
        </div>
        <div className="field">
          <span>{S.nextStepsTitle}</span>
          <ol className="steps">
            {target !== null && nextSteps(target, state.format).map((step) => <li key={step}>{step}</li>)}
          </ol>
        </div>
        <div className="actions">
          <button className="primary" onClick={openFolder}>{S.openFolder}</button>
          <button onClick={() => dispatch({ type: "restart" })}>{S.startOver}</button>
        </div>
      </div>
    </section>
  );
}
