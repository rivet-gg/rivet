import {
	Button,
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@rivet-gg/components";
import { Icon, faEllipsisH } from "@rivet-gg/icons";

export function BackendEnvironmentTableActions() {
	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<Button aria-haspopup="true" size="icon" variant="ghost">
					<Icon className="size-4" icon={faEllipsisH} />
					<span className="sr-only">Toggle menu</span>
				</Button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="end">
				<DropdownMenuItem>Manage</DropdownMenuItem>
			</DropdownMenuContent>
		</DropdownMenu>
	);
}
