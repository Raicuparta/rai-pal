import { RingProgress } from "@mantine/core";

type Props = {
	readonly percentage: number;
};

export function DownloadProgressRing(props: Props) {
	return (
		<RingProgress
			size={40}
			sections={[
				{
					value: props.percentage,
					color: props.percentage >= 100 ? "gray" : "green",
				},
			]}
		/>
	);
}
