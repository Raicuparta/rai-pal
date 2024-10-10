import { ProviderId } from "@api/bindings";
import {
	Icon,
	IconDeviceGamepad,
	IconBrandSteam,
	IconSquareLetterE,
	IconCircleLetterG,
	IconBrandXbox,
	IconBrandItch,
} from "@tabler/icons-react";

type Props = {
	readonly providerId: ProviderId;
};

const providerIcons: Record<ProviderId, Icon> = {
	Manual: IconDeviceGamepad,
	Steam: IconBrandSteam,
	Epic: IconSquareLetterE,
	Gog: IconCircleLetterG,
	HeroicGog: IconCircleLetterG,
	Xbox: IconBrandXbox,
	Itch: IconBrandItch,
	// Ubisoft: IconCircleLetterUFilled,
};

export function ProviderIcon(props: Props) {
	const IconComponent = providerIcons[props.providerId] ?? IconDeviceGamepad;
	return <IconComponent />;
}
