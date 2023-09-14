export function includesIgnoreCase(term: string, text: string) {
  return text.toLowerCase().includes(term.toLowerCase());
}

export function includesOneOf(term: string | undefined, texts: string[]) {
  if (!term) return true;

  return Boolean(texts.find((text) => includesIgnoreCase(term, text)));
}
