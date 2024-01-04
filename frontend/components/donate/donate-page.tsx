import {
	Container,
	Group,
	Stack,
	Card,
	Image,
	Text,
	Button,
	Divider,
} from "@mantine/core";
import {
	IconBrandGithubFilled,
	IconBrandItch,
	IconBrandPatreonFilled,
	IconBrandPaypalFilled,
} from "@tabler/icons-react";

export function DonatePage() {
	return (
		<Container
			size="xs"
			h="100%"
		>
			<Stack
				gap="xl"
				pb="md"
			>
				<Card>
					<Stack>
						<Group align="top">
							<Stack
								gap={0}
								bg="dark"
								justify="center"
								p="sm"
								style={{ flex: 1, borderRadius: 10 }}
							>
								<Image
									src="/images/donate/raicuparta.png"
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
							<Stack style={{ flex: 4 }}>
								<Text>
									Hello. I made Rai Pal. I also made other VR mods in the past,
									and am currently working in a universal VR mod for Unity
									games. If you like what I do, and would like to see more,
									please consider donating! You can also support me by buying
									one of my free mods on itch.io.
								</Text>
							</Stack>
						</Group>

						<Group>
							<Button
								component="a"
								href="https://www.patreon.com/raivr"
								target="_blank"
								color="pink"
								variant="light"
								style={{ flex: 1 }}
								leftSection={<IconBrandPatreonFilled />}
							>
								Patreon
							</Button>
							<Button
								component="a"
								href="https://paypal.me/raicuparta/5usd"
								target="_blank"
								color="blue"
								variant="light"
								style={{ flex: 1 }}
								leftSection={<IconBrandPaypalFilled />}
							>
								Paypal
							</Button>
							<Button
								component="a"
								href="https://raicuparta.itch.io/"
								target="_blank"
								color="red"
								variant="light"
								style={{ flex: 1 }}
								leftSection={<IconBrandItch />}
							>
								Itch.io
							</Button>
							<Button
								component="a"
								href="https://raicuparta.com"
								target="_blank"
								color="red"
								variant="light"
								style={{ flex: 1 }}
								leftSection={
									<Image
										src="/images/donate/raicuparta.png"
										radius="100%"
										height={20}
									/>
								}
							>
								raicuparta.com
							</Button>
						</Group>
					</Stack>
				</Card>
				<Divider />
				<Card>
					<Stack>
						<Group align="top">
							<Stack style={{ flex: 4 }}>
								<Text>
									Right now, the most likely reason for you to be using Rai Pal
									is to make it easier to manage Unreal Engine games for
									praydog&apos;s UEVR. You should consider supporting praydog on
									Patreon if you&apos;re excited about UEVR and want to continue
									seeing more of praydog&apos;s excellent work.
								</Text>
							</Stack>
							<Stack
								gap={0}
								bg="dark"
								justify="center"
								p="sm"
								style={{ flex: 1, borderRadius: 10 }}
							>
								<Image
									src="/images/donate/praydog.png"
									radius="100%"
								/>
								<Text
									fw={500}
									size="xl"
									ta="center"
								>
									praydog
								</Text>
							</Stack>
						</Group>
						<Group>
							<Button
								component="a"
								href="https://www.patreon.com/praydog"
								target="_blank"
								color="pink"
								variant="light"
								leftSection={<IconBrandPatreonFilled />}
								style={{ flex: 1 }}
							>
								praydog on Patreon
							</Button>

							<Button
								component="a"
								href="https://github.com/praydog/"
								target="_blank"
								color="grape"
								variant="light"
								leftSection={<IconBrandGithubFilled />}
								style={{ flex: 1 }}
							>
								praydog on GitHub
							</Button>
						</Group>
					</Stack>
				</Card>{" "}
				<Divider />
				<Card>
					<Stack>
						<Stack style={{ flex: 4 }}>
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
							<Button
								component="a"
								href="https://github.com/BepInEx"
								target="_blank"
								color="grape"
								variant="light"
								leftSection={<IconBrandGithubFilled />}
							>
								BepInEx on GitHub
							</Button>
							<Button
								component="a"
								href="https://www.patreon.com/pardeike"
								target="_blank"
								color="pink"
								variant="light"
								leftSection={<IconBrandPatreonFilled />}
							>
								Andreas Pardeike on Patreon
							</Button>
							<Button
								component="a"
								href="https://github.com/sinai-dev"
								target="_blank"
								color="grape"
								variant="light"
								leftSection={<IconBrandGithubFilled />}
							>
								sinai on GitHub
							</Button>
							<Button
								component="a"
								href="https://www.patreon.com/ManlyMarco"
								target="_blank"
								color="pink"
								variant="light"
								leftSection={<IconBrandPatreonFilled />}
							>
								ManlyMarco on Patreon
							</Button>
						</Stack>
					</Stack>
				</Card>
			</Stack>
		</Container>
	);
}
