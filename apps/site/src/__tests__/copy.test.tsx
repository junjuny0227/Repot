import { renderToStaticMarkup } from 'react-dom/server';
import { expect, test } from 'vitest';
import { ContentContext, content } from '../content';
import { CopyCommand, type CopyStatus, copyCommand } from '../sections/CopyCommand';
import { scrollState } from '../sections/ScrollingCode';

const BREW = 'brew install junjuny0227/tap/repot';

const render = (initialStatus: CopyStatus, locale: 'en' | 'ko' = 'en') =>
  renderToStaticMarkup(
    <ContentContext.Provider value={content(locale)}>
      <CopyCommand command={BREW} initialStatus={initialStatus} />
    </ContentContext.Provider>,
  );

test('copying succeeds only when the clipboard accepts the command', async () => {
  const written: string[] = [];
  const ok = { writeText: async (s: string) => void written.push(s) };
  expect(await copyCommand(ok, BREW)).toBe('copied');
  expect(written).toEqual([BREW]);
  const denied = { writeText: () => Promise.reject(new DOMException('denied', 'NotAllowedError')) };
  expect(await copyCommand(denied, BREW)).toBe('failed');
  expect(await copyCommand(undefined, BREW)).toBe('failed');
});

test('idle: the command, a labelled Copy button and an empty live region', () => {
  const html = render('idle');
  expect(html).toContain(`<code>${BREW}</code>`);
  expect(html).toContain('class="command idle"');
  expect(html).toContain('aria-label="Copy command"');
  expect(html).toContain('>Copy</span>');
  expect(html).toMatch(/<p class="copy-status" role="status"><\/p>/);
});

test('copied: the button says so and the announcement is quiet, not an error', () => {
  const html = render('copied');
  expect(html).toContain('class="command copied"');
  expect(html).toContain('>Copied</span>');
  expect(html).toContain('<span class="visually-hidden">Copied to the clipboard.</span>');
  expect(html).not.toContain('Could not copy');
});

test('failed: the live region explains what to do instead, in the page language', () => {
  const html = render('failed');
  expect(html).toContain('class="command failed"');
  expect(html).toContain('>Copy</span>');
  expect(html).toContain('Could not copy. Select the command and copy it manually.');
  expect(render('failed', 'ko')).toContain('복사하지 못했습니다.');
});

test('a scrolled command fades only the ends that hide text, and places the thumb', () => {
  // Fits: nothing hidden, no scrollbar.
  expect(scrollState(0, 300, 300)).toEqual({
    start: false,
    end: false,
    thumb: { offset: 0, size: 1 },
  });
  expect(scrollState(0, 300.5, 300).thumb.size).toBe(1);
  // At the start of a long command: only the end is hidden.
  expect(scrollState(0, 400, 200)).toEqual({
    start: false,
    end: true,
    thumb: { offset: 0, size: 0.5 },
  });
  // In the middle: both ends.
  expect(scrollState(100, 400, 200)).toEqual({
    start: true,
    end: true,
    thumb: { offset: 0.25, size: 0.5 },
  });
  // At the end, including a sub-pixel remainder: only the start.
  expect(scrollState(199.5, 400, 200)).toMatchObject({ start: true, end: false });
});

test('the server markup has no fade or scrollbar until the browser measures the command', () => {
  const html = render('idle');
  expect(html).toContain('<div class="command-text"><code>');
  expect(html).not.toContain('data-hidden');
  expect(html).not.toContain('command-scrollbar');
});
