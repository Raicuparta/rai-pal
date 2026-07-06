import { commands, AuthState } from "@api/bindings";
import { showAppNotification } from "@components/app-notifications";
import { useLocalization } from "@hooks/use-localization";
import { Avatar, Button, Menu, Text } from "@mantine/core";
import {
	IconBrandDiscordFilled,
	IconExternalLink,
	IconLogout2,
	IconUserCircle,
} from "@tabler/icons-react";
import { convertFileSrc } from "@tauri-apps/api/core";
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
	const [authState, setAuthState] = useState<AuthState>({
		isLoggedIn: false,
		avatarPath: null,
		userName: null,
	});

	const avatarUrl = authState.avatarPath
		? convertFileSrc(authState.avatarPath)
		: null;

	const refreshAuthState = useCallback(async () => {
		commands
			.getAuthState()
			.then((state) => {
				setAuthState(state);
			})
			.catch((error) => {
				console.error("Failed to read auth state:", error);
				showAppNotification(
					`Failed to read auth state: ${String(error)}`,
					"error",
				);
			});
	}, []);

	useEffect(() => {
		refreshAuthState();
	}, [refreshAuthState]);

	const userInitials = getInitials(authState.userName);

	const handleLogin = async () => {
		try {
			await commands.logIn();
			await refreshAuthState();
			commands.sendAnalyticsEvent("user_sign_in", null);
		} catch (error) {
			console.error("Login failed:", error);
			showAppNotification(`Sign-in failed: ${String(error)}`, "error");
		}
	};

	const handleLogout = async () => {
		try {
			await commands.logOut();
			await refreshAuthState();
		} catch (error) {
			console.error("Logout failed:", error);
			showAppNotification(`Sign-out failed: ${String(error)}`, "error");
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
