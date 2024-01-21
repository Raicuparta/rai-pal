import {
	Container,
	Group,
	Stack,
	Card,
	Image,
	Text,
	Divider,
} from "@mantine/core";
import {
	IconBrandGithubFilled,
	IconBrandItch,
	IconBrandPatreonFilled,
	IconBrandPaypalFilled,
} from "@tabler/icons-react";
import { DonateLinkButton } from "./donate-link-button";

export function DonatePage() {
	return (
		<Container
			size="xs"
			h="100%"
		>
			<Stack>
				<Stack
					gap="xl"
					pb="xs"
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
										Hello. I made Rai Pal. I also made other VR mods in the
										past, and am currently working on a universal VR mod for
										Unity games. If you like what I do, and would like to see
										more, please consider donating! You can also support me by
										buying one of my free mods on itch.io.
									</Text>
								</Stack>
							</Group>
							<Group>
								<DonateLinkButton
									color="blue"
									href="https://github.com/raicuparta/rai-pal"
									leftSection={<IconBrandGithubFilled />}
								>
									Star Rai Pal on GitHub
								</DonateLinkButton>
								<DonateLinkButton
									href="https://raicuparta.com"
									color="grape"
									leftSection={
										<Image
											src="/images/donate/raicuparta.png"
											radius="100%"
											height={20}
										/>
									}
								>
									raicuparta.com
								</DonateLinkButton>
							</Group>
							<Group>
								<DonateLinkButton
									href="https://www.patreon.com/raivr"
									color="pink"
									leftSection={<IconBrandPatreonFilled />}
								>
									Patreon
								</DonateLinkButton>
								<DonateLinkButton
									href="https://paypal.me/raicuparta/5usd"
									color="indigo"
									leftSection={<IconBrandPaypalFilled />}
								>
									Paypal
								</DonateLinkButton>
								<DonateLinkButton
									href="https://raicuparta.itch.io/"
									color="red"
									leftSection={<IconBrandItch />}
								>
									Itch.io
								</DonateLinkButton>
							</Group>
						</Stack>
					</Card>
					<Divider />
					<Card>
						<Stack>
							<Group align="top">
								<Stack style={{ flex: 4 }}>
									<Text>
										Right now, the most likely reason for you to be using Rai
										Pal is to make it easier to manage Unreal Engine games for
										praydog&apos;s UEVR. You should consider supporting praydog
										on Patreon if you&apos;re excited about UEVR and want to
										continue seeing more of praydog&apos;s excellent work.
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
								<DonateLinkButton
									href="https://www.patreon.com/praydog"
									color="pink"
									leftSection={<IconBrandPatreonFilled />}
								>
									praydog on Patreon
								</DonateLinkButton>

								<DonateLinkButton
									href="https://github.com/praydog/"
									color="blue"
									leftSection={<IconBrandGithubFilled />}
								>
									praydog on GitHub
								</DonateLinkButton>
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
									can&apos;t do that without the tools that other developers
									have created. Some of these people don&apos;t have donation
									links, but I&apos;m extremely grateful for their work.
								</Text>
							</Stack>
							<Stack>
								<DonateLinkButton
									href="https://github.com/BepInEx"
									color="blue"
									leftSection={<IconBrandGithubFilled />}
								>
									BepInEx on GitHub
								</DonateLinkButton>
								<DonateLinkButton
									href="https://www.patreon.com/pardeike"
									color="pink"
									leftSection={<IconBrandPatreonFilled />}
								>
									Andreas Pardeike on Patreon
								</DonateLinkButton>
								<DonateLinkButton
									href="https://github.com/sinai-dev"
									color="blue"
									leftSection={<IconBrandGithubFilled />}
								>
									sinai on GitHub
								</DonateLinkButton>
								<DonateLinkButton
									href="https://www.patreon.com/ManlyMarco"
									color="pink"
									leftSection={<IconBrandPatreonFilled />}
								>
									ManlyMarco on Patreon
								</DonateLinkButton>
							</Stack>
						</Stack>
					</Card>
				</Stack>
			</Stack>
		</Container>
	);
}
