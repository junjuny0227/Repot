import type { ReactNode } from 'react';

// SF Symbol-like glyphs drawn inline (no icon package). All are decorative: the
// element that uses them carries the accessible name. Category glyphs are copied
// from apps/desktop/src/screens/icons.tsx so the site matches the app sidebar.

export type ToolKey = 'packageManager' | 'runtime' | 'globalPackage' | 'editor' | 'git' | 'shell';
export type PrincipleKey = 'readonly' | 'secrets' | 'network' | 'review';

const GLYPHS: Record<ToolKey | PrincipleKey, ReactNode> = {
  packageManager: (
    <g fill="none" stroke="#fff" strokeWidth="1.3" strokeLinejoin="round">
      <path d="M8 2.6l5 2.5v5.8L8 13.4l-5-2.5V5.1z" />
      <path d="M3 5.1l5 2.5 5-2.5M8 7.6v5.8" />
    </g>
  ),
  runtime: (
    <g fill="none" stroke="#fff" strokeWidth="1.3" strokeLinejoin="round">
      <path d="M8 2.8l5.2 2.6L8 8 2.8 5.4z" />
      <path d="M2.8 8L8 10.6 13.2 8M2.8 10.6L8 13.2l5.2-2.6" />
    </g>
  ),
  globalPackage: (
    <g fill="none" stroke="#fff" strokeWidth="1.2">
      <circle cx="8" cy="8" r="5.4" />
      <ellipse cx="8" cy="8" rx="2.3" ry="5.4" />
      <path d="M2.6 8h10.8" />
    </g>
  ),
  editor: (
    <g fill="none" stroke="#fff" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
      <path d="M5.3 4.8L2.6 8l2.7 3.2M10.7 4.8L13.4 8l-2.7 3.2M9.1 3.6L6.9 12.4" />
    </g>
  ),
  git: (
    <g fill="none" stroke="#fff" strokeWidth="1.3">
      <circle cx="5" cy="3.9" r="1.4" />
      <circle cx="5" cy="12.1" r="1.4" />
      <circle cx="11" cy="6.2" r="1.4" />
      <path d="M5 5.3v5.4M11 7.6c0 2.2-6 1.4-6 3" />
    </g>
  ),
  shell: (
    <g fill="none" stroke="#fff" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3.4 5l3 3-3 3M8 11.4h4.6" />
    </g>
  ),
  // lock: nothing on the machine is touched
  readonly: (
    <g fill="none" stroke="#fff" strokeWidth="1.3" strokeLinejoin="round">
      <rect x="3.6" y="7" width="8.8" height="6.4" rx="1.6" />
      <path d="M5.6 7V5.3a2.4 2.4 0 0 1 4.8 0V7" />
    </g>
  ),
  // eye with a slash: values are hidden
  secrets: (
    <g fill="none" stroke="#fff" strokeWidth="1.3" strokeLinecap="round">
      <path d="M2.2 8c1.4-2.5 3.4-3.8 5.8-3.8s4.4 1.3 5.8 3.8c-1.4 2.5-3.4 3.8-5.8 3.8S3.6 10.5 2.2 8z" />
      <circle cx="8" cy="8" r="1.7" />
      <path d="M3.2 12.8l9.6-9.6" />
    </g>
  ),
  // wifi with a slash: no network
  network: (
    <g fill="none" stroke="#fff" strokeWidth="1.3" strokeLinecap="round">
      <path d="M2.4 6.6a8 8 0 0 1 11.2 0M4.4 8.8a5.2 5.2 0 0 1 7.2 0M6.4 11a2.3 2.3 0 0 1 3.2 0" />
      <path d="M3.2 13.2l9.6-10.4" />
    </g>
  ),
  // document with a magnifier: read it first
  review: (
    <g fill="none" stroke="#fff" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
      <path d="M10.4 6.4V3.2a1 1 0 0 0-1-1H4a1 1 0 0 0-1 1v9.6a1 1 0 0 0 1 1h3" />
      <path d="M5 5h3.4M5 7.4h2" />
      <circle cx="10.2" cy="10.2" r="2.1" />
      <path d="M11.8 11.8l1.6 1.6" />
    </g>
  ),
};

export function SymbolTile({ name }: { name: ToolKey | PrincipleKey }) {
  return (
    <span className={`tile tile-${name}`} aria-hidden="true">
      <svg viewBox="0 0 16 16" aria-hidden="true" focusable="false">
        {GLYPHS[name]}
      </svg>
    </span>
  );
}

export const WarningTriangle = () => (
  <svg className="symbol" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
    <path
      d="M6.9 2.1a1.27 1.27 0 0 1 2.2 0l5.6 9.9a1.27 1.27 0 0 1-1.1 1.9H2.4a1.27 1.27 0 0 1-1.1-1.9z"
      fill="currentColor"
    />
    <path d="M8 5.6v3.5" stroke="#fff" strokeWidth="1.5" strokeLinecap="round" />
    <circle cx="8" cy="11.4" r="0.85" fill="#fff" />
  </svg>
);

export const DownloadArrow = () => (
  <svg className="symbol" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
    <path
      d="M8 2.5v8M4.6 7.4L8 10.8l3.4-3.4M3 13.2h10"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  </svg>
);

export const Globe = () => (
  <svg className="symbol" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
    <circle cx="8" cy="8" r="6.4" fill="none" stroke="currentColor" strokeWidth="1.2" />
    <ellipse cx="8" cy="8" rx="2.7" ry="6.4" fill="none" stroke="currentColor" strokeWidth="1.1" />
    <path
      d="M1.8 8h12.4M2.9 4.8h10.2M2.9 11.2h10.2"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.1"
    />
  </svg>
);
