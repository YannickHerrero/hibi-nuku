// Hydrate dict bundles into IndexedDB on first run (or when versions change).
//
// Strategy: fetch /api/dict/manifest, compare each bundle's version
// against the stored meta value, and re-populate any that differ.
// The browser auto-gunzips the responses since the server sends
// `Content-Encoding: gzip`, so we read JSON directly.

import { getToken } from "@/lib/token";
import { openNukuDb, getMeta, setMeta } from "./idb";

import type { JmEntry, WkKanji, WkVocab } from "./idb";

export interface HydrateProgress {
  phase: "manifest" | "jmdict" | "wk" | "frequency" | "ready";
  loaded?: number;
  total?: number;
}

export interface DictManifest {
  jmdict: { url: string; size: number; sha256: string; version: string } | null;
  wk: { url: string; size: number; sha256: string; version: string } | null;
  frequency: { url: string; size: number; sha256: string; version: string } | null;
}

export async function hydrate(onProgress?: (p: HydrateProgress) => void) {
  onProgress?.({ phase: "manifest" });
  const manifest = await fetchManifest();

  if (manifest.jmdict) {
    const stored = await getMeta("jmdict");
    if (stored !== manifest.jmdict.version) {
      onProgress?.({ phase: "jmdict" });
      await hydrateJmdict(manifest.jmdict.url, onProgress);
      await setMeta("jmdict", manifest.jmdict.version);
    }
  }
  if (manifest.wk) {
    const stored = await getMeta("wk");
    if (stored !== manifest.wk.version) {
      onProgress?.({ phase: "wk" });
      await hydrateWk(manifest.wk.url, onProgress);
      await setMeta("wk", manifest.wk.version);
    }
  }
  if (manifest.frequency) {
    const stored = await getMeta("frequency");
    if (stored !== manifest.frequency.version) {
      onProgress?.({ phase: "frequency" });
      await hydrateFreq(manifest.frequency.url, onProgress);
      await setMeta("frequency", manifest.frequency.version);
    }
  }

  onProgress?.({ phase: "ready" });
}

async function fetchManifest(): Promise<DictManifest> {
  const resp = await fetch("/api/dict/manifest", {
    headers: authHeaders(),
  });
  if (!resp.ok) throw new Error(`manifest fetch: ${resp.status}`);
  return (await resp.json()) as DictManifest;
}

function authHeaders(): Record<string, string> {
  const t = getToken();
  return t ? { Authorization: `Bearer ${t}` } : {};
}

const BATCH = 1000;

async function hydrateJmdict(
  url: string,
  onProgress?: (p: HydrateProgress) => void,
) {
  const resp = await fetch(url, { headers: authHeaders() });
  if (!resp.ok) throw new Error(`jmdict fetch: ${resp.status}`);
  const bundle = (await resp.json()) as {
    version: string;
    entries: JmEntry[];
  };
  const db = await openNukuDb();

  // Build an inverted index keyed by every kanji and reading surface.
  const indexMap = new Map<string, number[]>();
  for (const e of bundle.entries) {
    for (const k of e.kanji ?? []) {
      let arr = indexMap.get(k);
      if (!arr) {
        arr = [];
        indexMap.set(k, arr);
      }
      arr.push(e.seq);
    }
    for (const r of e.readings ?? []) {
      let arr = indexMap.get(r);
      if (!arr) {
        arr = [];
        indexMap.set(r, arr);
      }
      arr.push(e.seq);
    }
  }

  await db.clear("jmdict_entries");
  await db.clear("jmdict_index");
  const total = bundle.entries.length + indexMap.size;
  let loaded = 0;

  for (let i = 0; i < bundle.entries.length; i += BATCH) {
    const tx = db.transaction("jmdict_entries", "readwrite");
    for (const e of bundle.entries.slice(i, i + BATCH)) tx.store.put(e);
    await tx.done;
    loaded += Math.min(BATCH, bundle.entries.length - i);
    onProgress?.({ phase: "jmdict", loaded, total });
  }

  const indexEntries = Array.from(indexMap.entries());
  for (let i = 0; i < indexEntries.length; i += BATCH) {
    const tx = db.transaction("jmdict_index", "readwrite");
    for (const [surface, seqs] of indexEntries.slice(i, i + BATCH)) {
      tx.store.put({ surface, seqs });
    }
    await tx.done;
    loaded += Math.min(BATCH, indexEntries.length - i);
    onProgress?.({ phase: "jmdict", loaded, total });
  }
}

async function hydrateWk(url: string, onProgress?: (p: HydrateProgress) => void) {
  const resp = await fetch(url, { headers: authHeaders() });
  if (!resp.ok) throw new Error(`wk fetch: ${resp.status}`);
  const bundle = (await resp.json()) as {
    version: string;
    user_level: number | null;
    kanji: Record<string, WkKanji>;
    vocab: Record<string, WkVocab>;
    vocab_by_kanji: Record<string, string[]>;
  };
  const db = await openNukuDb();
  await db.clear("wk_kanji");
  await db.clear("wk_vocab");
  await db.clear("wk_vocab_by_kanji");

  if (bundle.user_level !== null) {
    await setMeta("wk_user_level", String(bundle.user_level));
  }

  const kanjiEntries = Object.entries(bundle.kanji);
  const vocabEntries = Object.entries(bundle.vocab);
  const vbkEntries = Object.entries(bundle.vocab_by_kanji);
  const total = kanjiEntries.length + vocabEntries.length + vbkEntries.length;
  let loaded = 0;

  for (let i = 0; i < kanjiEntries.length; i += BATCH) {
    const tx = db.transaction("wk_kanji", "readwrite");
    for (const [character, v] of kanjiEntries.slice(i, i + BATCH)) {
      tx.store.put({ character, ...v });
    }
    await tx.done;
    loaded += Math.min(BATCH, kanjiEntries.length - i);
    onProgress?.({ phase: "wk", loaded, total });
  }

  for (let i = 0; i < vocabEntries.length; i += BATCH) {
    const tx = db.transaction("wk_vocab", "readwrite");
    for (const [vocab, v] of vocabEntries.slice(i, i + BATCH)) {
      tx.store.put({ vocab, ...v });
    }
    await tx.done;
    loaded += Math.min(BATCH, vocabEntries.length - i);
    onProgress?.({ phase: "wk", loaded, total });
  }

  for (let i = 0; i < vbkEntries.length; i += BATCH) {
    const tx = db.transaction("wk_vocab_by_kanji", "readwrite");
    for (const [kanji, vocab] of vbkEntries.slice(i, i + BATCH)) {
      tx.store.put({ kanji, vocab });
    }
    await tx.done;
    loaded += Math.min(BATCH, vbkEntries.length - i);
    onProgress?.({ phase: "wk", loaded, total });
  }
}

async function hydrateFreq(
  url: string,
  onProgress?: (p: HydrateProgress) => void,
) {
  const resp = await fetch(url, { headers: authHeaders() });
  if (!resp.ok) throw new Error(`frequency fetch: ${resp.status}`);
  const bundle = (await resp.json()) as {
    version: string;
    name: string;
    freq: Record<string, number>;
    readings?: Record<string, string>;
  };
  const db = await openNukuDb();
  await db.clear("frequency");

  const entries = Object.entries(bundle.freq);
  const total = entries.length;
  let loaded = 0;
  for (let i = 0; i < entries.length; i += BATCH) {
    const tx = db.transaction("frequency", "readwrite");
    for (const [term, rank] of entries.slice(i, i + BATCH)) {
      const reading = bundle.readings?.[term];
      tx.store.put({ term, rank, ...(reading ? { reading } : {}) });
    }
    await tx.done;
    loaded += Math.min(BATCH, entries.length - i);
    onProgress?.({ phase: "frequency", loaded, total });
  }
}
