export async function classify(source, row, { judge, threshold = 0.95 } = {}) {
  if (!Number.isFinite(threshold) || threshold < 0 || threshold > 1)
    throw new Error('Invalid threshold');
  if (!row.candidates.length) return { ...row, status: 'no-candidate' };
  const highs = row.candidates.filter((c) => c.tier === 'HIGH');
  if (highs.length === 1 && !highs[0].kindFlag)
    return { ...row, status: 'auto', picked: highs[0].id, probability: null, method: 'high-tier' };
  let outcome;
  try {
    outcome = await judge(source, row);
  } catch {
    return { ...row, status: 'incomplete', reason: 'Jeb decision unavailable; rerun to resume' };
  }
  const picked = row.candidates.find((c) => c.id === outcome.picked);
  if (
    (!picked && outcome.picked !== 'none') ||
    !Number.isFinite(outcome.probability) ||
    outcome.probability < 0 ||
    outcome.probability > 1
  )
    return { ...row, status: 'incomplete', reason: 'Invalid jeb decision' };
  return {
    ...row,
    ...outcome,
    method: 'jeb',
    status: picked && outcome.probability >= threshold && !picked.kindFlag ? 'auto' : 'review',
  };
}
