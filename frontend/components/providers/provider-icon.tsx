import { GameProviderId } from "@api/bindings";
import {
	Icon,
	IconDeviceGamepad,
	IconBrandSteam,
	IconSquareLetterE,
	IconCircleLetterG,
	IconBrandXbox,
	IconBrandItch,
} from "@tabler/icons-react";
import { ComponentProps } from "react";

interface Props extends ComponentProps<typeof IconDeviceGamepad> {
	readonly providerId: GameProviderId;
}

const providerIcons: Record<GameProviderId, Icon> = {
	Manual: IconDeviceGamepad,
	Steam: IconBrandSteam,
	Epic: IconSquareLetterE,
	Gog: IconCircleLetterG,
	Xbox: IconBrandXbox,
	Itch: IconBrandItch,
};

export function ProviderIcon({ providerId, ...props }: Props) {
	const IconComponent = providerIcons[providerId] ?? IconDeviceGamepad;
	return <IconComponent {...props} />;
}
