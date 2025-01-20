import { Button, WithTooltip } from "@rivet-gg/components";
import { Icon, faSave } from "@rivet-gg/icons";
import saveAs from "file-saver";

export function ActorDownloadLogsButton() {
	return (
		<WithTooltip
			content="Download logs"
			trigger={
				<Button
					className="ml-2 place-self-center"
					variant="ghost"
					aria-label="Download logs"
					size="icon-sm"
					onClick={() => {
						const lines = [];
						saveAs(
							new Blob([lines.join("\n")], {
								type: "text/plain;charset=utf-8",
							}),
							"logs.txt",
						);
					}}
				>
					<Icon icon={faSave} />
				</Button>
			}
		/>
	);
}
