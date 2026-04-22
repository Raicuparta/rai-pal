import { commands, DiscordAuthState } from "@api/bindings";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { Avatar, Button, Menu, Text } from "@mantine/core";
import { IconUserCircle } from "@tabler/icons-react";
import { useCallback, useEffect, useState } from "react";
import styles from "./tools.module.css";

async function getDiscordAuthState(): Promise<DiscordAuthState> {
	return commands.getDiscordAuthState() as Promise<DiscordAuthState>;
}

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
	const [authState, setAuthState] = useState<DiscordAuthState>({
		is_logged_in: false,
		avatar_file_path: null,
		user_name: null,
	});

	const refreshAuthState = useCallback(async () => {
		try {
			const state = await getDiscordAuthState();
			console.log("[Discord OAuth] Current auth state:", state);
			setAuthState(state);
		} catch (error) {
			console.error("[Discord OAuth] Failed to read auth state:", error);
		}
	}, []);

	useEffect(() => {
		refreshAuthState();
	}, [refreshAuthState]);

	const avatarUrl = authState.avatar_file_path
		? convertFileSrc(authState.avatar_file_path)
		: null;

	const userInitials = getInitials(authState.user_name);

	const handleLogin = async () => {
		console.log("[Discord OAuth] Login requested from user menu.");
		try {
			const result = await commands.startDiscordOauth();
			console.log("[Discord OAuth] Login completed:", result);
			await refreshAuthState();
		} catch (error) {
			console.error("[Discord OAuth] Login failed:", error);
		}
	};

	const handleLogout = async () => {
		console.log("[Discord OAuth] Logout requested from user menu.");
		try {
			await invoke<null>("logout_discord");
			console.log("[Discord OAuth] Logout completed.");
			await refreshAuthState();
		} catch (error) {
			console.error("[Discord OAuth] Logout failed:", error);
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
					{authState.is_logged_in ? (
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
			>
				{authState.is_logged_in && (
					<>
						<Text
							size="sm"
							c="dimmed"
							px="xs"
							pb={4}
						>
							Logged in as {authState.user_name ?? "Unknown user"}
						</Text>
						<Menu.Divider />
					</>
				)}
				{authState.is_logged_in ? (
					<Menu.Item onClick={handleLogout}>Log out</Menu.Item>
				) : (
					<Menu.Item onClick={handleLogin}>Log in</Menu.Item>
				)}
			</Menu.Dropdown>
		</Menu>
	);
}
