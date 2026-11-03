export function normUrl(value) {
  if (typeof value !== 'string' || !/^https?:\/\//.test(value)) return null;
  try {
    const url = new URL(value);
    const host = url.hostname.toLowerCase().replace(/^www\./, '');
    let path = decodeURIComponent(url.pathname).replace(/\/+$/, '').replace(/\.git$/, '');
    if (['github.com', 'gitlab.com', 'codeberg.org'].includes(host)) path = path.toLowerCase();
    return host + path;
  } catch { return null; }
}
export function repoKey(value) {
  const url = normUrl(value);
  if (!url) return null;
  let [host, ...parts] = url.split('/');
  if (host === 'raw.githubusercontent.com') host = 'github.com';
  if (host === 'api.github.com' && parts[0] === 'repos') { host = 'github.com'; parts.shift(); }
  if (!['github.com', 'gitlab.com', 'codeberg.org'].includes(host) || parts.length < 2 || ['features', 'topics', 'sponsors', 'orgs', 'settings'].includes(parts[0])) return null;
  if (host === 'gitlab.com' && parts[0] === 'api') return null;
  parts = host === 'gitlab.com' && parts.includes('-') ? parts.slice(0, parts.indexOf('-')) : parts.slice(0, 2);
  return `${host}/${parts.join('/').replace(/\.git$/, '').toLowerCase()}`;
}
export const nameKey = value => value.toLowerCase().replace(/[^a-z0-9]/g, '');
export const canonical = (target, value) => (target === 'scoop' && !value.includes('/') ? `main/${value}` : value).toLowerCase();
const unique = values => [...new Set(values.filter(Boolean))].sort();
export function record(id, names, urls, kind) {
  urls = urls.filter(value => typeof value === 'string');
  return { id, names: unique(names.filter(Boolean).map(nameKey)), urls, web: unique(urls.map(normUrl)), repos: unique(urls.map(repoKey)), kind };
}
export function strings(value) {
  if (typeof value === 'string') return [value];
  if (value && typeof value === 'object') return Object.values(value).flatMap(strings);
  return [];
}
