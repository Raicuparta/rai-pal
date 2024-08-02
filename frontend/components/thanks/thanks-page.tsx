import {
	Container,
	Group,
	Stack,
	Card,
	Image,
	Text,
	Box,
	Divider,
	Tooltip,
	MantineColor,
} from "@mantine/core";
import {
	IconBrandGithubFilled,
	IconBrandItch,
	IconBrandPatreonFilled,
	IconBrandPaypalFilled,
	IconHelpCircleFilled,
} from "@tabler/icons-react";
import { ThanksLinkButton } from "./thanks-link-button";
import { usePatrons } from "@hooks/use-patrons";
import styles from "./thanks.module.css";

function getRankingEmoji(ranking: number) {
	if (ranking == 1) return "🥇";
	if (ranking == 2) return "🥈";
	if (ranking == 3) return "🥉";
	return null;
}

function getRankingColor(ranking: number): MantineColor {
	if (ranking == 1) return "yellow";
	if (ranking == 2) return "gray";
	if (ranking == 3) return "red";
	return "dark";
}

export function ThanksPage() {
	const patrons = usePatrons();

	return (
		<Container
			h="100%"
			size="sm"
		>
			<Group h="100%">
				<Card
					flex={1}
					h="100%"
					style={{ overflowY: "auto" }}
				>
					<Stack>
						<Group align="top">
							<Stack
								gap={0}
								bg="dark"
								justify="center"
								p="sm"
								flex={1}
								style={{ borderRadius: 10 }}
							>
								<Image
									src="/images/thanks/raicuparta.png"
									radius="100%"
								/>
								<Text
									fw={500}
									size="xl"
									ta="center"
								>
									Raicuparta
								</Text>
							</Stack>
							<Stack flex={4}>
								<Text>
									Hello. I made Rai Pal. I also made other VR mods in the past,
									and am currently working on a universal VR mod for Unity
									games. If you like what I do, and would like to see more,
									please consider donating! You can also support me by buying
									one of my free mods on itch.io.
								</Text>
							</Stack>
						</Group>
						<Group>
							<ThanksLinkButton
								color="blue"
								href="https://github.com/raicuparta/rai-pal"
								leftSection={<IconBrandGithubFilled />}
							>
								Star Rai Pal on GitHub
							</ThanksLinkButton>
							<ThanksLinkButton
								href="https://raicuparta.com"
								color="grape"
								leftSection={
									<Image
										src="/images/thanks/raicuparta.png"
										radius="100%"
										height={20}
									/>
								}
							>
								raicuparta.com
							</ThanksLinkButton>
						</Group>
						<Group>
							<ThanksLinkButton
								href="https://www.patreon.com/raivr"
								color="pink"
								leftSection={<IconBrandPatreonFilled />}
							>
								Patreon
							</ThanksLinkButton>
							<ThanksLinkButton
								href="https://paypal.me/raicuparta/5usd"
								color="indigo"
								leftSection={<IconBrandPaypalFilled />}
							>
								Paypal
							</ThanksLinkButton>
							<ThanksLinkButton
								href="https://raicuparta.itch.io/"
								color="red"
								leftSection={<IconBrandItch />}
							>
								Itch.io
							</ThanksLinkButton>
						</Group>
						<Divider />
						<Stack>
							<Text
								fw={500}
								size="xl"
							>
								Other modders
							</Text>
							<Text>
								Rai Pal is meant to help you manage game modding, and we
								can&apos;t do that without the tools that other developers have
								created. Some of these people don&apos;t have donation links,
								but I&apos;m extremely grateful for their work.
							</Text>
						</Stack>
						<Stack>
							<ThanksLinkButton
								href="https://www.patreon.com/praydog"
								color="pink"
								leftSection={<IconBrandPatreonFilled />}
							>
								praydog on Patreon
							</ThanksLinkButton>
							<ThanksLinkButton
								href="https://github.com/BepInEx"
								color="blue"
								leftSection={<IconBrandGithubFilled />}
							>
								BepInEx on GitHub
							</ThanksLinkButton>
							<ThanksLinkButton
								href="https://www.patreon.com/pardeike"
								color="pink"
								leftSection={<IconBrandPatreonFilled />}
							>
								Andreas Pardeike on Patreon
							</ThanksLinkButton>
							<ThanksLinkButton
								href="https://github.com/sinai-dev"
								color="blue"
								leftSection={<IconBrandGithubFilled />}
							>
								sinai on GitHub
							</ThanksLinkButton>
							<ThanksLinkButton
								href="https://www.patreon.com/ManlyMarco"
								color="pink"
								leftSection={<IconBrandPatreonFilled />}
							>
								ManlyMarco on Patreon
							</ThanksLinkButton>
						</Stack>
					</Stack>
				</Card>
				{patrons.length > 0 && (
					<Stack
						h="100%"
						w={200}
					>
						<Group align="top">
							<ThanksLinkButton
								pos="relative"
								size="lg"
								href="https://www.patreon.com/raivr"
								color="pink"
								flex={1}
							>
								<Stack gap={0}>
									<div>Patreon</div>
									<div>Leaderboard</div>
								</Stack>
								<Box
									pos="absolute"
									right={0}
									top={0}
								>
									<Tooltip
										ta="center"
										label={
											<>
												<Text>Ranked by total lifetime donation amount.</Text>
												<Text>
													If you don&apos;t see yourself here, it&apos;s because
													your Patreon profile is private.
												</Text>
											</>
										}
									>
										<IconHelpCircleFilled />
									</Tooltip>
								</Box>
							</ThanksLinkButton>
						</Group>
						<Card
							h="100%"
							style={{ overflowY: "scroll" }}
						>
							<Stack>
								{patrons.map((patron) => (
									<Group
										key={patron.ranking}
										pos="relative"
									>
										<Box c={getRankingColor(patron.ranking)}>
											<img
												className={styles.patronAvatar}
												src={patron.imageUrl}
											/>
										</Box>
										{patron.ranking <= 3 && (
											<Text
												size="xl"
												className={styles.patronMedal}
											>
												{getRankingEmoji(patron.ranking)}
											</Text>
										)}
										<Text
											flex={1}
											className={styles.patronName}
											fw={patron.ranking <= 3 ? "bold" : "normal"}
											c={patron.ranking <= 3 ? "white" : undefined}
										>
											{patron.name}
										</Text>
									</Group>
								))}
							</Stack>
						</Card>
					</Stack>
				)}
			</Group>
		</Container>
	);
}
