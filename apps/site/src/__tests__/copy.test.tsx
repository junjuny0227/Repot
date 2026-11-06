import { renderToStaticMarkup } from 'react-dom/server';
import { expect, test } from 'vitest';
import { ContentContext, content } from '../content';
import { CopyCommand, type CopyStatus, copyCommand } from '../sections/CopyCommand';

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
