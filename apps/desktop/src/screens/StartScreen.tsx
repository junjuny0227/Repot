import type { Dispatch } from 'react';
import appIcon from '../../app-icon.svg?raw';
import type { Action, State } from '../state';
import { osLabel, S } from '../strings';
import { LanguageSelect } from './LanguageSelect';
import { TargetSelect } from './TargetSelect';

export function StartScreen({
  state,
  dispatch,
  onScan,
  onOpen,
}: {
  state: State;
  dispatch: Dispatch<Action>;
  onScan: () => void;
  onOpen: () => void;
}) {
  const ready = state.source !== null && state.target !== null;
  return (
    <section className="screen start">
      <div className="titlebar" data-tauri-drag-region="deep" />
      <div className="form-page">
        <header className="hero">
          {/* Inline markup instead of <img>: the CSP blocks data: URLs that Vite would emit. */}
          <span
            className="app-icon"
            aria-hidden="true"
            dangerouslySetInnerHTML={{ __html: appIcon }}
          />
          <h1>{S.appTitle}</h1>
          <p className="muted">{S.startIntro}</p>
        </header>
        <div className="group">
          <div className="row">
            <span className="row-label">{S.currentOs}</span>
            <span className="row-value">
              {state.source === null ? S.detectingOs : osLabel(state.source)}
            </span>
          </div>
          <TargetSelect
            className="row"
            value={state.target}
            onChange={(target) => dispatch({ type: 'setTarget', target })}
          />
        </div>
        <div className="actions">
          <button className="large" disabled={!ready} onClick={onOpen}>
            {S.openFile}
          </button>
          <button className="primary large" disabled={!ready} onClick={onScan}>
            {S.scanButton}
          </button>
        </div>
      </div>
      <footer className="start-footer">
        <LanguageSelect />
      </footer>
    </section>
  );
}
