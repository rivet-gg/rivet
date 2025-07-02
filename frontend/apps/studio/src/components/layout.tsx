import { cn, DocsSheet } from "@rivet-gg/components";
import { useManagerInspector } from "@rivet-gg/components/actors";
import { Header as RivetHeader, NavItem } from "@rivet-gg/components/header";
import {
	faCheck,
	faGithub,
	faSpinnerThird,
	faTriangleExclamation,
	Icon,
} from "@rivet-gg/icons";
import { Link } from "@tanstack/react-router";
import type { PropsWithChildren, ReactNode } from "react";

interface RootProps {
	children: ReactNode;
}

const Root = ({ children }: RootProps) => {
	return <div className={cn("flex min-h-screen flex-col")}>{children}</div>;
};

const Main = ({ children }: RootProps) => {
	return (
		<main className="bg-background flex flex-1 flex-col h-full min-h-0 relative">
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
	const ws = useManagerInspector();

	if (ws.isConnecting) {
		return (
			<p className="animate-in fade-in">
				Connecting to{" "}
				<span className="underline underline-offset-2">
					localhost:6420
				</span>
				<Icon icon={faSpinnerThird} className="animate-spin ml-2" />
			</p>
		);
	}

	if (ws.isDisconnected) {
		return (
			<p className="text-red-500 animate-shake">
				Couldn't connect to{" "}
				<span className="underline underline-offset-2">
					localhost:6420
				</span>
				<Icon icon={faTriangleExclamation} className="ml-2" />
			</p>
		);
	}

	if (ws.isConnected) {
		return (
			<p className="text-primary animate-in fade-in">
				Connected to{" "}
				<span className="underline underline-offset-2">
					localhost:6420
				</span>
				<Icon icon={faCheck} className="ml-2" />
			</p>
		);
	}
}

const Header = () => {
	return (
		<RivetHeader
			className="bg-stripes"
			logo={
				<>
					<img src="/logo.svg" alt="Rivet.gg" className="h-6" />{" "}
					Studio
					<div className="text-xs  font-mono text-muted-foreground">
						<ConnectionStatus />
					</div>
				</>
			}
			links={
				<>
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
