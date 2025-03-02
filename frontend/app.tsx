import { useData } from "@hooks/use-data";
import { AppNotifications } from "@components/app-notifications";
import { useAppUpdater } from "@hooks/use-app-updater";
import { AppTabs } from "@components/app-tabs";
import { TranslationContext } from "@hooks/use-translations";
import { useEffect, useState } from "react";
import { isLanguageCode, LanguageCode } from "./translations/translations";
import { locale } from "@tauri-apps/plugin-os";

function App() {
	const [language, setLanguage] = useState<LanguageCode>("en");
	useAppUpdater();
	useData();

	useEffect(() => {
		locale().then((localeCode) => {
			const languageCode = localeCode?.split("-")[0];
			if (!languageCode) {
				console.error(
					"Invalid locale, couldn't get a language code out of it",
					localeCode,
				);
				return;
			}

			if (isLanguageCode(languageCode)) {
				setLanguage(languageCode);
			}
		});
	}, []);

	return (
		<TranslationContext value={language}>
			<AppNotifications />
			<AppTabs />
		</TranslationContext>
	);
}

export default App;
