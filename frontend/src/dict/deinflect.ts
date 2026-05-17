// Compact Yomitan-style deinflection (client-side) — mirrors the
// pragmatic subset shipped on the backend (see backend/src/tokenize/
// deinflect.rs). Used by the popup when the user selects a surface
// that doesn't match the pre-tokenised JMDict entry.

interface Rule {
  in: string;
  out: string;
  // Tags this rule produces; lookups filter entries by overlap with
  // an entry's JMDict rule tags. Empty = no constraint.
  rulesOut: string[];
}

const RULES: Rule[] = [
  // polite
  { in: "ました", out: "ます", rulesOut: ["v5"] },
  { in: "ません", out: "ます", rulesOut: ["v5"] },
  { in: "ます", out: "る", rulesOut: ["v1"] },
  // ichidan
  { in: "て", out: "る", rulesOut: ["v1"] },
  { in: "た", out: "る", rulesOut: ["v1"] },
  { in: "ない", out: "る", rulesOut: ["v1"] },
  { in: "なかった", out: "る", rulesOut: ["v1"] },
  // godan -ku / -gu / -su
  { in: "いた", out: "く", rulesOut: ["v5k"] },
  { in: "いて", out: "く", rulesOut: ["v5k"] },
  { in: "かない", out: "く", rulesOut: ["v5k"] },
  { in: "きます", out: "く", rulesOut: ["v5k"] },
  { in: "いだ", out: "ぐ", rulesOut: ["v5g"] },
  { in: "いで", out: "ぐ", rulesOut: ["v5g"] },
  { in: "がない", out: "ぐ", rulesOut: ["v5g"] },
  { in: "ぎます", out: "ぐ", rulesOut: ["v5g"] },
  { in: "した", out: "す", rulesOut: ["v5s"] },
  { in: "して", out: "す", rulesOut: ["v5s"] },
  { in: "さない", out: "す", rulesOut: ["v5s"] },
  { in: "します", out: "す", rulesOut: ["v5s"] },
  // godan -tsu / -ru / -u (t-base)
  { in: "った", out: "つ", rulesOut: ["v5t"] },
  { in: "って", out: "つ", rulesOut: ["v5t"] },
  { in: "たない", out: "つ", rulesOut: ["v5t"] },
  { in: "ちます", out: "つ", rulesOut: ["v5t"] },
  { in: "った", out: "る", rulesOut: ["v5r"] },
  { in: "って", out: "る", rulesOut: ["v5r"] },
  { in: "らない", out: "る", rulesOut: ["v5r"] },
  { in: "ります", out: "る", rulesOut: ["v5r"] },
  { in: "った", out: "う", rulesOut: ["v5u"] },
  { in: "って", out: "う", rulesOut: ["v5u"] },
  { in: "わない", out: "う", rulesOut: ["v5u"] },
  { in: "います", out: "う", rulesOut: ["v5u"] },
  // godan -bu / -mu / -nu
  { in: "んだ", out: "ぶ", rulesOut: ["v5b"] },
  { in: "んで", out: "ぶ", rulesOut: ["v5b"] },
  { in: "ばない", out: "ぶ", rulesOut: ["v5b"] },
  { in: "びます", out: "ぶ", rulesOut: ["v5b"] },
  { in: "んだ", out: "む", rulesOut: ["v5m"] },
  { in: "んで", out: "む", rulesOut: ["v5m"] },
  { in: "まない", out: "む", rulesOut: ["v5m"] },
  { in: "みます", out: "む", rulesOut: ["v5m"] },
  // suru / kuru / i-adjectives
  { in: "しました", out: "する", rulesOut: ["vs-i"] },
  { in: "して", out: "する", rulesOut: ["vs-i"] },
  { in: "した", out: "する", rulesOut: ["vs-i"] },
  { in: "しない", out: "する", rulesOut: ["vs-i"] },
  { in: "します", out: "する", rulesOut: ["vs-i"] },
  { in: "来た", out: "来る", rulesOut: ["vk"] },
  { in: "来て", out: "来る", rulesOut: ["vk"] },
  { in: "きた", out: "くる", rulesOut: ["vk"] },
  { in: "きて", out: "くる", rulesOut: ["vk"] },
  { in: "かった", out: "い", rulesOut: ["adj-i"] },
  { in: "くない", out: "い", rulesOut: ["adj-i"] },
  { in: "くて", out: "い", rulesOut: ["adj-i"] },
];

export interface Candidate {
  stem: string;
  rulesOut: Set<string>;
}

export function deinflect(surface: string): Candidate[] {
  const out: Candidate[] = [{ stem: surface, rulesOut: new Set() }];
  const seen = new Set([surface]);
  for (const r of RULES) {
    if (surface.endsWith(r.in)) {
      const stem = surface.slice(0, -r.in.length) + r.out;
      if (!seen.has(stem)) {
        seen.add(stem);
        out.push({ stem, rulesOut: new Set(r.rulesOut) });
      }
    }
  }
  return out;
}
