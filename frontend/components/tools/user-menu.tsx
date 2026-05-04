import { commands, DiscordAuthState } from "@api/bindings";
import { showAppNotification } from "@components/app-notifications";
import { useLocalization } from "@hooks/use-localization";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Avatar, Button, Menu, Text } from "@mantine/core";
import {
	IconBrandDiscordFilled,
	IconExternalLink,
	IconLogout2,
	IconUserCircle,
} from "@tabler/icons-react";
import { useCallback, useEffect, useState } from "react";

function getInitials(name: string | null): string {
	const words = name?.match(/\S+/g);
	if (!words) return "?";

	const initials =
		words.length === 1
			? (words[0]?.match(/[a-z0-9]/gi) || []).join("")
			: words.map((w) => w.match(/[a-z0-9]/i)?.[0]).join("");

	return initials.slice(0, 2).toUpperCase() || "?";
}

export function UserMenu() {
	const t = useLocalization("userMenu");
	const [authState, setAuthState] = useState<DiscordAuthState>({
		isLoggedIn: false,
		avatarFilePath: null,
		userName: null,
	});

	const refreshAuthState = useCallback(async () => {
		try {
			const state = await commands.getAuthState();
			console.log("Current auth state:", state);
			setAuthState(state);
		} catch (error) {
			console.error("Failed to read auth state:", error);
			showAppNotification(
				`Failed to read Discord auth state: ${String(error)}`,
				"error",
			);
		}
	}, []);

	useEffect(() => {
		refreshAuthState();
	}, [refreshAuthState]);

	const avatarUrl = authState.avatarFilePath
		? convertFileSrc(authState.avatarFilePath)
		: null;

	const userInitials = getInitials(authState.userName);

	const handleLogin = async () => {
		console.log("Login requested from user menu.");
		try {
			const result = await commands.logIn();
			console.log("Login completed:", result);
			await refreshAuthState();
		} catch (error) {
			console.error("Login failed:", error);
			showAppNotification(`Discord sign-in failed: ${String(error)}`, "error");
		}
	};

	const handleLogout = async () => {
		console.log("Logout requested from user menu.");
		try {
			await commands.logOut();
			console.log("Logout completed.");
			await refreshAuthState();
		} catch (error) {
			console.error("Logout failed:", error);
			showAppNotification(`Discord sign-out failed: ${String(error)}`, "error");
		}
	};

	return (
		<Menu
			closeOnItemClick={true}
			keepMounted={true}
			withOverlay={false}
		>
			<Menu.Target>
				<Button
					variant="filled"
					color="dark"
					fz="md"
				>
					{authState.isLoggedIn ? (
						<Avatar
							radius="xl"
							bd="2px solid white"
							src={avatarUrl}
							size="sm"
							bg="black"
						>
							{userInitials}
						</Avatar>
					) : (
						<IconUserCircle color="white" />
					)}
				</Button>
			</Menu.Target>
			<Menu.Dropdown
				p="xs"
				bg="dark"
				maw={250}
			>
				{authState.isLoggedIn ? (
					<>
						<Menu.Label>{authState.userName ?? t("unknownUser")}</Menu.Label>
						<Menu.Item
							onClick={handleLogout}
							leftSection={<IconLogout2 />}
							color="red"
						>
							{t("logOut")}
						</Menu.Item>
					</>
				) : (
					<>
						<Menu.Item
							onClick={handleLogin}
							leftSection={<IconBrandDiscordFilled />}
							rightSection={<IconExternalLink />}
							bg="violet"
							c="white"
						>
							{t("signInWithDiscord")}
						</Menu.Item>
						<Text
							size="sm"
							c="dimmed"
							pt="xs"
						>
							{t("discordAccessNote")}
						</Text>
					</>
				)}
			</Menu.Dropdown>
		</Menu>
	);
}
