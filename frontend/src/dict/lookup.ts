// Read-side lookups against the hydrated IndexedDB stores.

import { openNukuDb, type JmEntry, type WkKanji } from "./idb";

export interface JmHit extends JmEntry {
  freqRank?: number;
}

export async function lookupSurface(surface: string): Promise<JmEntry[]> {
  const db = await openNukuDb();
  const indexRow = await db.get("jmdict_index", surface);
  if (!indexRow) return [];
  const seqs = indexRow.seqs;
  const tx = db.transaction("jmdict_entries", "readonly");
  const out: JmEntry[] = [];
  for (const seq of seqs) {
    const e = await tx.store.get(seq);
    if (e) out.push(e);
  }
  return out;
}

export async function wkKanji(c: string): Promise<WkKanji | undefined> {
  const db = await openNukuDb();
  const row = await db.get("wk_kanji", c);
  if (!row) return undefined;
  return row;
}

export async function wkVocabByKanji(c: string): Promise<string[]> {
  const db = await openNukuDb();
  const row = await db.get("wk_vocab_by_kanji", c);
  return row?.vocab ?? [];
}

export async function frequencyRank(term: string): Promise<number | undefined> {
  const db = await openNukuDb();
  const row = await db.get("frequency", term);
  return row?.rank;
}
