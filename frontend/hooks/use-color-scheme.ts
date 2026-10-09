import { useEffect, useEffectEvent } from "react";
import { useComputedColorScheme, useMantineColorScheme } from "@mantine/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppSettingSingle } from "./use-app-setting-single";

export function useColorScheme() {
	const [colorScheme] = useAppSettingSingle("colorScheme");
	const { setColorScheme } = useMantineColorScheme();
	const computedColorScheme = useComputedColorScheme("light");

	const applyColorScheme = useEffectEvent(
		(scheme: "auto" | "dark" | "light") => {
			setColorScheme(scheme);

			getCurrentWindow()
				.setTheme(scheme === "auto" ? null : scheme)
				.catch((error) => {
					console.error(`Failed to set window theme: ${String(error)}`);
				});
		},
	);

	useEffect(() => {
		const scheme = (colorScheme ?? "Auto").toLowerCase() as
			"auto" | "dark" | "light";
		applyColorScheme(scheme);
	}, [colorScheme]);

	useEffect(() => {
		document.documentElement.style.colorScheme = computedColorScheme;
	}, [computedColorScheme]);
}
