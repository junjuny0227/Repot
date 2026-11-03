import { expect, test } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import App from "../App";
import { VERSION, type NavigatorLike } from "../downloads";

const render = (nav: NavigatorLike) => renderToStaticMarkup(<App nav={nav} />);
const mac = { userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.1 Safari/605.1.15", maxTouchPoints: 0 };
const windows = { userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:145.0) Gecko/20100101 Firefox/145.0" };
const iphone = { userAgent: "Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148" };

test("one h1, and every section is titled by an h2", () => {
  const html = render(mac);
  expect(html.match(/<h1[\s>]/g)).toHaveLength(1);
  const sections = html.match(/<section[^>]*aria-labelledby="([^"]+)"/g) ?? [];
  expect(sections.length).toBe(5);
  for (const s of sections) {
    const id = s.match(/aria-labelledby="([^"]+)"/)![1];
    expect(html).toMatch(new RegExp(`<h[12] id="${id}"`));
  }
});

test("every image has alt text", () => {
  const imgs = render(mac).match(/<img[^>]*>/g) ?? [];
  expect(imgs.length).toBeGreaterThan(0);
  for (const img of imgs) expect(img).toMatch(/ alt="/);
});

test("download links all point at the current version's release assets", () => {
  for (const nav of [mac, windows, iphone]) {
    const hrefs = [...render(nav).matchAll(/href="([^"]*releases\/download[^"]*)"/g)].map((m) => m[1]);
    expect(hrefs.length).toBeGreaterThan(0);
    for (const h of hrefs) expect(h.startsWith(`https://github.com/junjuny0227/Repot/releases/download/v${VERSION}/`)).toBe(true);
  }
});

test("the hero offers the detected platform's installer", () => {
  expect(render(mac)).toContain(`v${VERSION}/Repot_${VERSION}_aarch64.dmg`);
  const win = render(windows);
  const hero = win.slice(0, win.indexOf('id="how"'));
  expect(hero).toContain("-setup.exe");
  expect(hero).not.toContain(".dmg");
  // Phones get pointed at the download list instead of a file.
  const phone = render(iphone);
  expect(phone.slice(0, phone.indexOf('id="how"'))).not.toContain("releases/download");
});
