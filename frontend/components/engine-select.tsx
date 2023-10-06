import {
	SegmentedControlData,
	TypedSegmentedControl,
	TypedSegmentedControlProps,
} from "@components/installed-games/typed-segmented-control";
import { GameEngineBrand } from "@api/bindings";

const engineOptions: SegmentedControlData<GameEngineBrand>[] = [
	{ label: "Any Engine", value: "" },
	{ label: "Unity", value: "Unity" },
	{ label: "Unreal", value: "Unreal" },
	{ label: "Godot", value: "Godot" },
];

type Props = Omit<TypedSegmentedControlProps<GameEngineBrand>, "data">;

export function EngineSelect(props: Props) {
	return (
		<TypedSegmentedControl
			data={engineOptions}
			onChange={props.onChange}
			value={props.value}
		/>
	);
}
