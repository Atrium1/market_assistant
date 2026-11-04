const TR_FOLD: Record<string, string> = {
  ç: "c",
  ğ: "g",
  ı: "i",
  ö: "o",
  ş: "s",
  ü: "u",
};

/** Lowercase with Turkish rules and fold diacritics, so "IŞIK", "ışık" and "isik" compare equal. */
export function normalizeTr(value: string): string {
  return value
    .toLocaleLowerCase("tr-TR")
    .replace(/[çğıöşü]/g, (ch) => TR_FOLD[ch])
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "");
}

const ONES = ["ı", "i", "si", "ü", "ü", "i", "sı", "si", "i", "u"];
const TENS = ["", "u", "si", "u", "ı", "si", "ı", "i", "i", "ı"];

/** Third-person possessive suffix for a number as read aloud: 12 → "si", 30 → "u", 100 → "ü". */
export function possessiveSuffixTr(n: number): string {
  if (n === 0) return "ı";
  if (n % 10 !== 0) return ONES[n % 10];
  if (n % 100 !== 0) return TENS[Math.floor(n / 10) % 10];
  if (n % 1000 !== 0) return "ü";
  if (n % 1_000_000 !== 0) return "i";
  return "u";
}
