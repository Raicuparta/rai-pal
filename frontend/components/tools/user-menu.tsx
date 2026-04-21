import { commands } from "@api/bindings";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { Avatar, Button, Menu } from "@mantine/core";
import { IconUserCircle } from "@tabler/icons-react";
import { useCallback, useEffect, useMemo, useState } from "react";

type DiscordAuthState = {
	is_logged_in: boolean;
	avatar_file_path: string | null;
};

async function getDiscordAuthState(): Promise<DiscordAuthState> {
	return invoke<DiscordAuthState>("get_discord_auth_state");
}

export function UserMenu() {
	const [authState, setAuthState] = useState<DiscordAuthState>({
		is_logged_in: false,
		avatar_file_path: null,
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
		void refreshAuthState();
	}, [refreshAuthState]);

	const avatarUrl = useMemo(() => {
		if (!authState.avatar_file_path) return null;
		return convertFileSrc(authState.avatar_file_path);
	}, [authState.avatar_file_path]);

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
					px={8}
				>
					{authState.is_logged_in ? (
						<Avatar
							size={22}
							radius="xl"
							src={avatarUrl}
						/>
					) : (
						<IconUserCircle />
					)}
				</Button>
			</Menu.Target>
			<Menu.Dropdown
				p="xs"
				bg="dark"
			>
				{authState.is_logged_in ? (
					<Menu.Item onClick={handleLogout}>Log out</Menu.Item>
				) : (
					<Menu.Item onClick={handleLogin}>Log in</Menu.Item>
				)}
			</Menu.Dropdown>
		</Menu>
	);
}
