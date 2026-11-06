import { type PointerEvent, useEffect, useRef, useState } from 'react';

export type ScrollState = {
  /** Text is hidden past the left end, so that end fades out. */
  start: boolean;
  /** Text is hidden past the right end. */
  end: boolean;
  /** The thumb's left edge and width, as fractions of the track. */
  thumb: { offset: number; size: number };
};

const FITS: ScrollState = { start: false, end: false, thumb: { offset: 0, size: 1 } };

/** What a one-line command scrolled to `scrollLeft` hides, and where its scrollbar thumb sits. */
export function scrollState(
  scrollLeft: number,
  scrollWidth: number,
  clientWidth: number,
): ScrollState {
  const range = scrollWidth - clientWidth;
  // 1px slack: fractional widths can leave a sub-pixel remainder at either end.
  if (range <= 1) return FITS;
  return {
    start: scrollLeft > 1,
    end: range - scrollLeft > 1,
    thumb: { offset: scrollLeft / scrollWidth, size: clientWidth / scrollWidth },
  };
}

/**
 * A command on one line that scrolls sideways. Each end that hides text fades out, and a thin
 * overlay scrollbar appears while the pointer is over the command box (see `.command-box` in
 * site.css). The native scrollbar is hidden: a classic one would take height only in boxes
 * whose command overflows. The thumb can be dragged; trackpads and Shift-wheel scroll as usual.
 */
export function ScrollingCode({ children }: { children: string }) {
  const scroller = useRef<HTMLElement>(null);
  const [state, setState] = useState<ScrollState>(FITS);
  const measure = () => {
    const el = scroller.current;
    if (el) setState(scrollState(el.scrollLeft, el.scrollWidth, el.clientWidth));
  };
  // Re-measure when the box changes width (window resize, font load).
  useEffect(() => {
    const el = scroller.current;
    if (!el || typeof ResizeObserver === 'undefined') return;
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const drag = useRef<{ x: number; scrollLeft: number } | null>(null);
  const onPointerDown = (e: PointerEvent<HTMLDivElement>) => {
    const el = scroller.current;
    if (!el) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    drag.current = { x: e.clientX, scrollLeft: el.scrollLeft };
  };
  const onPointerMove = (e: PointerEvent<HTMLDivElement>) => {
    const el = scroller.current;
    if (!el || !drag.current) return;
    // The thumb moves clientWidth/scrollWidth as fast as the text.
    el.scrollLeft =
      drag.current.scrollLeft + ((e.clientX - drag.current.x) * el.scrollWidth) / el.clientWidth;
  };
  const endDrag = () => {
    drag.current = null;
  };

  const overflows = state.thumb.size < 1;
  return (
    <div
      className="command-text"
      data-hidden-start={state.start || undefined}
      data-hidden-end={state.end || undefined}
    >
      <code ref={scroller} onScroll={measure}>
        {children}
      </code>
      {overflows && (
        <div className="command-scrollbar" aria-hidden="true">
          <div
            className="command-thumb"
            style={{
              left: `${state.thumb.offset * 100}%`,
              width: `${state.thumb.size * 100}%`,
            }}
            onPointerDown={onPointerDown}
            onPointerMove={onPointerMove}
            onPointerUp={endDrag}
            onPointerCancel={endDrag}
          />
        </div>
      )}
    </div>
  );
}
