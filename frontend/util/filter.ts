export function includesIgnoreCase(term: string, text: string) {
  return text.toLowerCase().includes(term.toLowerCase());
}

export function includesOneOf(term: string, texts: string[]) {
  return Boolean(texts.find((text) => includesIgnoreCase(term, text)));
}
