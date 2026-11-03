import type { ReactNode } from "react";
import type { Category } from "../types";

// SF Symbol-like glyphs drawn inline (no icon package). All are decorative:
// the element that uses them carries the accessible name.

function Svg({ className, children, size = 16 }: { className?: string; children: ReactNode; size?: number }) {
  return (
    <svg className={className} width={size} height={size} viewBox="0 0 16 16" aria-hidden="true" focusable="false">
      {children}
    </svg>
  );
}

export const CheckCircle = ({ className = "symbol" }: { className?: string }) => (
  <Svg className={className}>
    <circle cx="8" cy="8" r="7" fill="currentColor" />
    <path d="M4.9 8.3l2.1 2.1 4.2-4.6" fill="none" stroke="#fff" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
  </Svg>
);

export const MinusCircle = ({ className = "symbol" }: { className?: string }) => (
  <Svg className={className}>
    <circle cx="8" cy="8" r="6.4" fill="none" stroke="currentColor" strokeWidth="1.2" />
    <path d="M5.2 8h5.6" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
  </Svg>
);

export const ExclamationCircle = ({ className = "symbol" }: { className?: string }) => (
  <Svg className={className}>
    <circle cx="8" cy="8" r="7" fill="currentColor" />
    <path d="M8 4.4v4.4" stroke="#fff" strokeWidth="1.6" strokeLinecap="round" />
    <circle cx="8" cy="11.3" r="0.95" fill="#fff" />
  </Svg>
);

export const ClockCircle = ({ className = "symbol" }: { className?: string }) => (
  <Svg className={className}>
    <circle cx="8" cy="8" r="7" fill="currentColor" />
    <path d="M8 4.3V8l2.6 1.6" fill="none" stroke="#fff" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
  </Svg>
);

export const WarningTriangle = ({ className = "symbol" }: { className?: string }) => (
  <Svg className={className}>
    <path d="M6.9 2.1a1.27 1.27 0 0 1 2.2 0l5.6 9.9a1.27 1.27 0 0 1-1.1 1.9H2.4a1.27 1.27 0 0 1-1.1-1.9z" fill="currentColor" />
    <path className="glyph" d="M8 5.6v3.5" stroke="#fff" strokeWidth="1.5" strokeLinecap="round" />
    <circle className="glyph-dot" cx="8" cy="11.4" r="0.85" fill="#fff" />
  </Svg>
);

export const InfoCircle = ({ className = "symbol" }: { className?: string }) => (
  <Svg className={className}>
    <circle cx="8" cy="8" r="7" fill="currentColor" />
    <circle cx="8" cy="4.9" r="0.95" fill="#fff" />
    <path d="M8 7.3v4.2" stroke="#fff" strokeWidth="1.6" strokeLinecap="round" />
  </Svg>
);

export const Duplicate = ({ className = "symbol" }: { className?: string }) => (
  <Svg className={className}>
    <rect x="5" y="5" width="8.5" height="8.5" rx="2" fill="none" stroke="currentColor" strokeWidth="1.3" />
    <path d="M3 10.5V4.5A2 2 0 0 1 5 2.5h6" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
  </Svg>
);

export const NoSign = ({ className = "symbol" }: { className?: string }) => (
  <Svg className={className}>
    <circle cx="8" cy="8" r="6.2" fill="none" stroke="currentColor" strokeWidth="1.4" />
    <path d="M3.7 12.3l8.6-8.6" stroke="currentColor" strokeWidth="1.4" />
  </Svg>
);

export const Globe = () => (
  <Svg className="symbol globe">
    <circle cx="8" cy="8" r="6.4" fill="none" stroke="currentColor" strokeWidth="1.2" />
    <ellipse cx="8" cy="8" rx="2.7" ry="6.4" fill="none" stroke="currentColor" strokeWidth="1.1" />
    <path d="M1.8 8h12.4M2.9 4.8h10.2M2.9 11.2h10.2" fill="none" stroke="currentColor" strokeWidth="1.1" />
  </Svg>
);

export const Magnifier = () => (
  <Svg>
    <circle cx="6.8" cy="6.8" r="4.6" fill="none" stroke="currentColor" strokeWidth="1.5" />
    <path d="M10.3 10.3l3.7 3.7" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
  </Svg>
);

export const Chevrons = () => (
  <Svg className="chevrons">
    <path d="M5.2 6.6L8 3.9l2.8 2.7M5.2 9.4L8 12.1l2.8-2.7" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
  </Svg>
);

export const Spinner = ({ large = false }: { large?: boolean }) => (
  <Svg className={large ? "spinner large" : "spinner"}>
    {Array.from({ length: 8 }, (_, i) => (
      <rect key={i} x="7.25" y="0.8" width="1.5" height="4.2" rx="0.75" fill="currentColor" opacity={0.22 + (i / 7) * 0.78} transform={`rotate(${i * 45} 8 8)`} />
    ))}
  </Svg>
);

export const DocIcon = () => (
  <Svg className="symbol doc">
    <path d="M4 1.8h5l3.2 3.2v8.2a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V2.8a1 1 0 0 1 1-1z" fill="none" stroke="currentColor" strokeWidth="1.1" />
    <path d="M9 1.8V5h3.2" fill="none" stroke="currentColor" strokeWidth="1.1" />
  </Svg>
);

export const TerminalIcon = () => (
  <Svg className="symbol doc">
    <rect x="1.8" y="2.8" width="12.4" height="10.4" rx="2" fill="none" stroke="currentColor" strokeWidth="1.1" />
    <path d="M4.6 6.2l2 1.8-2 1.8M8 10h3.2" fill="none" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" strokeLinejoin="round" />
  </Svg>
);

// White glyphs that sit on a colored rounded square, like System Settings.
const GLYPHS: Record<Category | "all", ReactNode> = {
  all: (
    <g fill="#fff">
      <rect x="3" y="3" width="4.3" height="4.3" rx="1.1" /><rect x="8.7" y="3" width="4.3" height="4.3" rx="1.1" />
      <rect x="3" y="8.7" width="4.3" height="4.3" rx="1.1" /><rect x="8.7" y="8.7" width="4.3" height="4.3" rx="1.1" />
    </g>
  ),
  packageManager: (
    <g fill="none" stroke="#fff" strokeWidth="1.3" strokeLinejoin="round">
      <path d="M8 2.6l5 2.5v5.8L8 13.4l-5-2.5V5.1z" /><path d="M3 5.1l5 2.5 5-2.5M8 7.6v5.8" />
    </g>
  ),
  runtime: (
    <g fill="none" stroke="#fff" strokeWidth="1.3" strokeLinejoin="round">
      <path d="M8 2.8l5.2 2.6L8 8 2.8 5.4z" /><path d="M2.8 8L8 10.6 13.2 8M2.8 10.6L8 13.2l5.2-2.6" />
    </g>
  ),
  globalPackage: (
    <g fill="none" stroke="#fff" strokeWidth="1.2">
      <circle cx="8" cy="8" r="5.4" /><ellipse cx="8" cy="8" rx="2.3" ry="5.4" /><path d="M2.6 8h10.8" />
    </g>
  ),
  editor: (
    <g fill="none" stroke="#fff" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
      <path d="M5.3 4.8L2.6 8l2.7 3.2M10.7 4.8L13.4 8l-2.7 3.2M9.1 3.6L6.9 12.4" />
    </g>
  ),
  git: (
    <g fill="none" stroke="#fff" strokeWidth="1.3">
      <circle cx="5" cy="3.9" r="1.4" /><circle cx="5" cy="12.1" r="1.4" /><circle cx="11" cy="6.2" r="1.4" />
      <path d="M5 5.3v5.4M11 7.6c0 2.2-6 1.4-6 3" />
    </g>
  ),
  shell: (
    <g fill="none" stroke="#fff" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3.4 5l3 3-3 3M8 11.4h4.6" />
    </g>
  ),
};

export function CategoryIcon({ category }: { category: Category | "all" }) {
  return (
    <span className={`category-icon icon-${category}`} aria-hidden="true">
      <svg width="14" height="14" viewBox="0 0 16 16" focusable="false">{GLYPHS[category]}</svg>
    </span>
  );
}
