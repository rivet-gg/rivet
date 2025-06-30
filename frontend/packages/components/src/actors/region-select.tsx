import { Combobox } from "@rivet-gg/components";
import { ActorRegion } from "./actor-region";
import { useQuery } from "@tanstack/react-query";
import { useManagerQueries } from "./manager-queries-context";

interface RegionSelectProps {
	onValueChange: (value: string) => void;
	value: string;
}

export function RegionSelect({ onValueChange, value }: RegionSelectProps) {
	const { data = [] } = useQuery(useManagerQueries().regionsQueryOptions());

	const regions = [
		{
			label: "Automatic (Recommended)",
			value: "",
			region: { id: "", name: "Automatic (Recommended)" },
		},
		...data.map((region) => {
			return {
				label: <ActorRegion regionId={region.id} showLabel />,
				value: region.id,
				region,
			};
		}),
	];

	return (
		<Combobox
			placeholder="Choose a region..."
			options={regions}
			value={value}
			onValueChange={onValueChange}
			filter={(option, searchMixed) => {
				const search = searchMixed.toLowerCase();
				return (
					option.region.id.includes(search) ||
					option.region.name.includes(search)
				);
			}}
			className="w-full"
		/>
	);
}
