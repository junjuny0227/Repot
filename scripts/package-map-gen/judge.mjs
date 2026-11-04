import { createHash } from 'node:crypto';
import { mkdir, open, readFile, rename, rm, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

export async function readKey({
  env = process.env,
  envPath = new URL('../../.env', import.meta.url),
} = {}) {
  if (env.JEB_AI_HUB_KEY?.trim()) return env.JEB_AI_HUB_KEY.trim();
  let text;
  try {
    text = await readFile(envPath, 'utf8');
  } catch {
    throw new Error('JEB_AI_HUB_KEY is required');
  }
  const value = text.match(/^\s*(?:export\s+)?JEB_AI_HUB_KEY\s*=\s*(.*?)\s*$/m)?.[1];
  const key =
    value?.match(/^(["'])(.*?)\1\s*(?:#.*)?$/)?.[2] ?? value?.replace(/\s+#.*$/, '').trim();
  if (!key) throw new Error('JEB_AI_HUB_KEY is required');
  return key;
}
export function buildRequest(source, row) {
  const criteria = Object.fromEntries(
    row.candidates.map((c, i) => [
      String(i + 1),
      `${c.id}: ${c.name || c.fields?.PackageName || c.id}. ${c.description || c.fields?.ShortDescription || c.fields?.Description || ''}. Homepage ${c.homepage || c.fields?.PackageUrl || c.urls?.[0] || ''}. Kind ${['cli', 'gui'].includes(c.kind) ? c.kind : 'unknown'}`,
    ]),
  );
  criteria[String(row.candidates.length + 1)] = 'none of these is the same software';
  return {
    model: 'jeb',
    state: {
      kind: source.brew_kind,
      token: source.id,
      name: source.name || source.names,
      desc: source.description || source.desc || '',
      homepage: source.homepage || source.urls?.[0] || '',
      source_repo_key: source.repos,
      target: row.target,
    },
    questions: {
      q: {
        type: 'choice',
        instructions:
          'Choose the candidate that is the same software as the source package, not merely a functional alternative. Respect product/component, release channel and major-version constraints. Select none if no candidate is the same software. Source formula/cask kind is a hint, not proof of CLI/GUI.',
        criteria,
      },
    },
  };
}
function decode(body, row) {
  const a = body?.answers?.q,
    choice = String(a?.choice),
    index = Number(choice) - 1;
  const probability = Array.isArray(a?.probabilities)
    ? a.probabilities[index]
    : a?.probabilities?.[choice];
  if (
    !/^\d+$/.test(choice) ||
    !Number.isInteger(index) ||
    index < 0 ||
    index > row.candidates.length ||
    typeof probability !== 'number' ||
    !Number.isFinite(probability) ||
    probability < 0 ||
    probability > 1
  )
    throw new Error('Invalid jeb response');
  return {
    picked: index === row.candidates.length ? 'none' : row.candidates[index].id,
    probability,
    method: 'jeb',
  };
}
async function readJson(path, fallback) {
  try {
    return JSON.parse(await readFile(path, 'utf8'));
  } catch (error) {
    if (error.code === 'ENOENT') return fallback;
    throw new Error('Invalid jeb checkpoint or cache');
  }
}
async function save(path, value) {
  await writeFile(path + '.tmp', JSON.stringify(value, null, 2) + '\n');
  await rename(path + '.tmp', path);
}
// A serial queue deliberately stays below the maximum of two in-flight calls.
export function createJudge({
  cacheDir = '.cache/package-map-gen',
  fetch = globalThis.fetch,
  sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms)),
  now = () => new Date(),
  maxRequests = 1000,
  creditFloor = 100,
  offline = false,
  key,
  env,
  envPath,
} = {}) {
  let queue = Promise.resolve(),
    attempts = 0;
  const stats = { requests: 0, cached: 0, remainingCredits: null };
  async function judge(source, row) {
    const payload = buildRequest(source, row),
      hash = createHash('sha256').update(JSON.stringify(payload)).digest('hex');
    const responses = join(cacheDir, 'responses'),
      cachePath = join(responses, hash + '.json');
    const cached = await readJson(cachePath, null);
    if (cached) {
      const result = decode(cached, row);
      stats.cached++;
      return { ...result, cached: true };
    }
    if (offline) throw new Error('Offline jeb response cache missing');
    const checkpointPath = join(cacheDir, 'checkpoint.json'),
      day = now().toISOString().slice(0, 10);
    let checkpoint = await readJson(checkpointPath, {
      day,
      attempts: 0,
      remainingCredits: null,
      stopped: null,
    });
    if (
      !checkpoint ||
      !/^\d{4}-\d{2}-\d{2}$/.test(checkpoint.day) ||
      !Number.isSafeInteger(checkpoint.attempts) ||
      checkpoint.attempts < 0 ||
      checkpoint.attempts > 1000 ||
      (checkpoint.remainingCredits !== null &&
        (!Number.isFinite(checkpoint.remainingCredits) || checkpoint.remainingCredits < 0))
    )
      throw new Error('Invalid jeb checkpoint');
    if (checkpoint.day !== day)
      checkpoint = { day, attempts: 0, remainingCredits: null, stopped: null };
    const secret = key ?? (await readKey({ env, envPath }));
    await mkdir(responses, { recursive: true });
    for (let retry = 0; retry < 7; retry++) {
      if (attempts >= maxRequests || checkpoint.attempts >= 1000)
        throw new Error('Jeb request limit reached');
      if (
        checkpoint.stopped ||
        (checkpoint.remainingCredits !== null && checkpoint.remainingCredits < creditFloor)
      )
        throw new Error('Jeb quota stopped; resume after quota reset');
      attempts++;
      checkpoint.attempts++;
      stats.requests++;
      // Persist the reservation before transport; crashes must not refund attempts.
      checkpoint.stopped = 'request interrupted';
      await save(checkpointPath, checkpoint);
      let response;
      try {
        response = await fetch('https://jeb.https.gsmsv.site/v1/systemone', {
          method: 'POST',
          headers: { Authorization: `Bearer ${secret}`, 'Content-Type': 'application/json' },
          body: JSON.stringify(payload),
          signal: AbortSignal.timeout(240_000),
        });
      } catch {
        throw new Error('Jeb transport failed; quota may have been consumed');
      }
      const header = response.headers.get('x-ratelimit-remaining-credits');
      const credits = header?.trim() ? Number(header) : NaN;
      checkpoint.remainingCredits = Number.isFinite(credits) && credits >= 0 ? credits : null;
      stats.remainingCredits = checkpoint.remainingCredits;
      checkpoint.stopped =
        checkpoint.remainingCredits === null
          ? 'invalid quota header'
          : credits < creditFloor
            ? 'credit floor'
            : null;
      await save(checkpointPath, checkpoint);
      if (checkpoint.remainingCredits === null)
        throw new Error('Jeb quota header missing or invalid');
      if (response.status === 503) {
        if (checkpoint.stopped) throw new Error('Jeb credit floor reached');
        if (retry === 6) throw new Error('Jeb 503 retry limit reached');
        await sleep(Math.min(60_000, 1000 * 2 ** retry));
        continue;
      }
      if (!response.ok) throw new Error(`Jeb HTTP ${response.status}`);
      let body;
      try {
        body = await response.json();
      } catch {
        throw new Error('Invalid jeb response');
      }
      const result = decode(body, row);
      // Store only validated choice/probability, never response text or headers.
      const choice = String(
        result.picked === 'none'
          ? row.candidates.length + 1
          : row.candidates.findIndex((c) => c.id === result.picked) + 1,
      );
      await save(cachePath, {
        answers: { q: { choice, probabilities: { [choice]: result.probability } } },
      });
      return { ...result, cached: false };
    }
  }
  async function locked(source, row) {
    if (offline) return judge(source, row);
    await mkdir(cacheDir, { recursive: true });
    const path = join(cacheDir, 'judge.lock');
    let lock;
    try {
      lock = await open(path, 'wx');
    } catch {
      throw new Error('Jeb cache is locked; another run may be active');
    }
    try {
      return await judge(source, row);
    } finally {
      await lock.close();
      await rm(path);
    }
  }
  const client = (source, row) => {
    const task = queue.then(() => locked(source, row));
    queue = task.catch(() => {});
    return task;
  };
  Object.defineProperty(client, 'stats', { get: () => ({ ...stats }) });
  return client;
}
