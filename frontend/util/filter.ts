import { TableColumn } from "@components/table/table-head";

export function includesIgnoreCase(term: string, text: string) {
	return text.toLowerCase().includes(term.toLowerCase());
}

export function includesOneOf(term: string | undefined, texts: string[]) {
	if (!term) return true;

	return Boolean(texts.find((text) => includesIgnoreCase(term, text)));
}

export function filterGame<TKey extends string, TGame>(
	game: TGame,
	hiddenValues: Record<string, (string | null)[]>,
	columns: TableColumn<TKey, TGame>[],
) {
	return (
		columns.findIndex((column) => {
			return (
				column.getFilterValue &&
				hiddenValues[column.id] &&
				hiddenValues[column.id].length > 0 &&
				hiddenValues[column.id].includes(column.getFilterValue(game))
			);
		}) === -1
	);
}
