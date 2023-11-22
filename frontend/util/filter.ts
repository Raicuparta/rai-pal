import { TableColumn } from "@components/table/table-head";

export function includesIgnoreCase(term: string, text: string) {
	return text.toLowerCase().includes(term.toLowerCase());
}

export function includesOneOf(term: string | undefined, texts: string[]) {
	if (!term) return true;

	return Boolean(texts.find((text) => includesIgnoreCase(term, text)));
}

export function filterGame<TGame>(
	game: TGame,
	filter: Record<string, string>,
	columns: TableColumn<TGame>[],
) {
	return (
		columns.findIndex((column) => {
			const getValueFunction = column.getFilterValue ?? column.getSortValue;

			return (
				getValueFunction &&
				filter[column.id] &&
				filter[column.id] !== getValueFunction(game)
			);
		}) === -1
	);
}
