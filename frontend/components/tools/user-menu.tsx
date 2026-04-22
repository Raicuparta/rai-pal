import { commands, DiscordAuthState } from "@api/bindings";
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
	const [authState, setAuthState] = useState<DiscordAuthState>({
		is_logged_in: false,
		avatar_file_path: null,
		user_name: null,
	});

	const refreshAuthState = useCallback(async () => {
		try {
			const state = await commands.getAuthState();
			console.log("Current auth state:", state);
			setAuthState(state);
		} catch (error) {
			console.error("Failed to read auth state:", error);
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
		console.log("Login requested from user menu.");
		try {
			const result = await commands.logIn();
			console.log("Login completed:", result);
			await refreshAuthState();
		} catch (error) {
			console.error("Login failed:", error);
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
				{authState.is_logged_in ? (
					<>
						<Menu.Label>{authState.user_name ?? "Unknown user"}</Menu.Label>
						<Menu.Item
							onClick={handleLogout}
							leftSection={<IconLogout2 />}
							color="red"
						>
							Log out
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
							Sign in with Discord
						</Menu.Item>
						<Text
							size="sm"
							c="dimmed"
							pt="xs"
							maw={200}
						>
							Mods can use this to access your Discord username, avatar, roles,
							etc.
						</Text>
					</>
				)}
			</Menu.Dropdown>
		</Menu>
	);
}
