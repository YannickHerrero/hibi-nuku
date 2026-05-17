// IndexedDB schema + open helper.

import { openDB, type DBSchema, type IDBPDatabase } from "idb";

export interface JmEntry {
  seq: number;
  kanji: string[];
  readings: string[];
  senses: { pos: string[]; glosses: string[] }[];
  rules?: string[];
}

export interface WkKanji {
  level: number;
  meaning: string;
  reading: string;
  reading_type: string;
}

export interface WkVocab {
  level: number;
  meanings: string[];
  readings: string[];
}

interface NukuDB extends DBSchema {
  jmdict_entries: {
    key: number; // seq
    value: JmEntry;
  };
  jmdict_index: {
    key: string; // kanji or reading surface
    value: { surface: string; seqs: number[] };
  };
  wk_kanji: {
    key: string; // character
    value: WkKanji & { character: string };
  };
  wk_vocab: {
    key: string;
    value: WkVocab & { vocab: string };
  };
  wk_vocab_by_kanji: {
    key: string;
    value: { kanji: string; vocab: string[] };
  };
  frequency: {
    key: string; // term
    value: { term: string; rank: number; reading?: string };
  };
  meta: {
    key: string; // 'jmdict' | 'wk' | 'frequency' | 'wk_user_level'
    value: { name: string; value: string };
  };
}

const DB_NAME = "nuku-dict";
const DB_VERSION = 1;

export async function openNukuDb(): Promise<IDBPDatabase<NukuDB>> {
  return openDB<NukuDB>(DB_NAME, DB_VERSION, {
    upgrade(db) {
      db.createObjectStore("jmdict_entries", { keyPath: "seq" });
      db.createObjectStore("jmdict_index", { keyPath: "surface" });
      db.createObjectStore("wk_kanji", { keyPath: "character" });
      db.createObjectStore("wk_vocab", { keyPath: "vocab" });
      db.createObjectStore("wk_vocab_by_kanji", { keyPath: "kanji" });
      db.createObjectStore("frequency", { keyPath: "term" });
      db.createObjectStore("meta", { keyPath: "name" });
    },
  });
}

export async function getMeta(name: string): Promise<string | undefined> {
  const db = await openNukuDb();
  const v = await db.get("meta", name);
  return v?.value;
}

export async function setMeta(name: string, value: string) {
  const db = await openNukuDb();
  await db.put("meta", { name, value });
}
