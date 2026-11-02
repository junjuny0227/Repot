/** Tauri rejects with plain strings; JS failures throw Errors. */
export const errorMessage = (e: unknown): string => (e instanceof Error ? e.message : String(e));
