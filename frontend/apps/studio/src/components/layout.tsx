import { Button, cn, DocsSheet } from "@rivet-gg/components";
import { Header as RivetHeader, NavItem } from "@rivet-gg/components/header";
import {
	faCheck,
	faDiscord,
	faGithub,
	faLink,
	faSpinnerThird,
	faTriangleExclamation,
	Icon,
} from "@rivet-gg/icons";
import { Link } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import type { PropsWithChildren, ReactNode } from "react";
import { useManagerQueries } from "@rivet-gg/components/actors";

interface RootProps {
	children: ReactNode;
}

const Root = ({ children }: RootProps) => {
	return <div className={cn("flex min-h-screen flex-col")}>{children}</div>;
};

const Main = ({ children }: RootProps) => {
	return (
		<main className="bg-background flex flex-1 flex-col h-full min-h-0 min-w-0 relative">
			{children}
		</main>
	);
};

const VisibleInFull = ({ children }: PropsWithChildren) => {
	return (
		<div className="relative min-h-screen max-h-screen grid grid-rows-[auto,1fr]">
			{children}
		</div>
	);
};

function ConnectionStatus() {
	const { setToken, endpoint, ...queries } = useManagerQueries();
	const { isLoading, isError, isSuccess } = useQuery(
		queries.managerStatusQueryOptions(),
	);

	if (!queries.managerStatusQueryOptions().enabled) {
		return null;
	}

	if (isLoading) {
		return (
			<p className="animate-in fade-in">
				Connecting to{" "}
				<span className="underline underline-offset-2">{endpoint}</span>
				<Icon icon={faSpinnerThird} className="animate-spin ml-2" />
			</p>
		);
	}

	if (isError) {
		return (
			<p className="text-red-500">
				Couldn't connect to{" "}
				<span className="underline underline-offset-2">{endpoint}</span>
				<Icon icon={faTriangleExclamation} className="ml-2" />
				<Button
					variant="outline"
					size="xs"
					className="ml-2 text-foreground"
					onClick={() => setToken("", "")}
				>
					<Icon icon={faLink} />
					Reconnect
				</Button>
			</p>
		);
	}

	if (isSuccess) {
		return (
			<p className="text-primary animate-in fade-in">
				Connected to{" "}
				<span className="underline underline-offset-2">{endpoint}</span>
				<Icon icon={faCheck} className="ml-2" />
			</p>
		);
	}
}

const Header = () => {
	return (
		<RivetHeader
			className="bg-stripes border-b-2 border-b-primary/90"
			logo={
				<>
					<div className="flex items-center gap-2">
						<img src="/logo.svg" alt="Rivet.gg" className="h-6" />{" "}
						Studio
					</div>
					<div className="text-xs  font-mono text-muted-foreground">
						<ConnectionStatus />
					</div>
				</>
			}
			links={
				<>
					<NavItem asChild>
						<a href="http://rivet.gg/discord">
							<Icon icon={faDiscord} />
						</a>
					</NavItem>
					<NavItem asChild>
						<a href="https://github.com/rivet-gg/rivet">
							<Icon icon={faGithub} />
						</a>
					</NavItem>
					<DocsSheet
						path={"https://actorcore.org/overview"}
						title="Documentation"
					>
						<NavItem className="cursor-pointer">
							Documentation
						</NavItem>
					</DocsSheet>
					<NavItem asChild>
						<Link
							to="."
							search={(old) => ({ ...old, modal: "feedback" })}
						>
							Feedback
						</Link>
					</NavItem>
				</>
			}
		/>
	);
};

const Footer = () => {
	return null;
};

export { Root, Main, Header, Footer, VisibleInFull };
