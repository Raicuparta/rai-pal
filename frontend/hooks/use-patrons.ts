import { atom, useAtom } from "jotai";
import { useEffect } from "react";

export type Patron = {
	readonly ranking: number;
	readonly imageUrl: string;
	readonly name: string;
};

type PatronsDatabase = {
	activePatrons: Patron[];
};

const PATRONS_JSON_URL =
	"https://raicuparta.github.io/patron-fetcher/patrons.json";

const patronsAtom = atom<Patron[]>([]);

export const usePatrons = () => {
	const [patrons, setPatrons] = useAtom(patronsAtom);

	useEffect(() => {
		if (patrons.length > 0) return;
		fetch(PATRONS_JSON_URL)
			.then((response) => response.json())
			.then((database: PatronsDatabase) => {
				setPatrons(database.activePatrons);
			})
			.catch((error) => {
				console.error("Failed to fetch patrons", error);
			});
	}, [patrons.length, setPatrons]);

	return patrons;
};
