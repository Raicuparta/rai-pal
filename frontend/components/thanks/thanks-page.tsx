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
import { useLocalization } from "@hooks/use-localization";

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
	const t = useLocalization("thanksPage");
	const patrons = usePatrons();

	return (
		<Container
			h="100%"
			size="sm"
			p={0}
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
								<Text>{t("intro")}</Text>
							</Stack>
						</Group>
						<Group>
							<ThanksLinkButton
								color="blue"
								href="https://github.com/raicuparta/rai-pal"
								leftSection={<IconBrandGithubFilled />}
							>
								{t("starRaiPalOnGitHub")}
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
								{t("otherModdersTitle")}
							</Text>
							<Text>{t("otherModdersDescription")}</Text>
						</Stack>
						<Stack>
							<ThanksLinkButton
								href="https://www.patreon.com/praydog"
								color="pink"
								leftSection={<IconBrandPatreonFilled />}
							>
								{t("modderOnWebsite", {
									modderName: "praydog",
									website: "Patreon",
								})}
							</ThanksLinkButton>
							<ThanksLinkButton
								href="https://github.com/BepInEx"
								color="blue"
								leftSection={<IconBrandGithubFilled />}
							>
								{t("modderOnWebsite", {
									modderName: "BepInEx",
									website: "GitHub",
								})}
							</ThanksLinkButton>
							<ThanksLinkButton
								href="https://www.patreon.com/pardeike"
								color="pink"
								leftSection={<IconBrandPatreonFilled />}
							>
								{t("modderOnWebsite", {
									modderName: "Andreas Pardeike",
									website: "Patreon",
								})}
							</ThanksLinkButton>
							<ThanksLinkButton
								href="https://github.com/sinai-dev"
								color="blue"
								leftSection={<IconBrandGithubFilled />}
							>
								{t("modderOnWebsite", {
									modderName: "sinai",
									website: "GitHub",
								})}
							</ThanksLinkButton>
							<ThanksLinkButton
								href="https://www.patreon.com/ManlyMarco"
								color="pink"
								leftSection={<IconBrandPatreonFilled />}
							>
								{t("modderOnWebsite", {
									modderName: "ManlyMarco",
									website: "Patreon",
								})}
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
								<Box style={{ textWrap: "wrap" }}>
									{t("patreonLeaderboard")}
								</Box>
								<Box
									pos="absolute"
									right={0}
									top={0}
								>
									<Tooltip
										ta="center"
										label={
											<>
												<Text>{t("rankedByPatreonDonationAmount")}</Text>
												<Text>{t("patreonProfilePrivateNotice")}</Text>
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
