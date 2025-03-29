import { connectionStateAtom } from "@/stores/manager";
import { cn, ShimmerLine } from "@rivet-gg/components";
import { Header as RivetHeader, NavItem } from "@rivet-gg/components/header";
import {
	Icon,
	faBluesky,
	faDiscord,
	faGithub,
	faXTwitter,
} from "@rivet-gg/icons";
import { Link } from "@tanstack/react-router";
import { useAtomValue } from "jotai";
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
		<div className="relative min-h-screen grid grid-rows-[auto,1fr]">
			{children}
		</div>
	);
};

const Header = () => {
	const connectionStatus = useAtomValue(connectionStateAtom);
	return (
		<RivetHeader
			logo={<img src="/logo.svg" alt="Rivet.gg" className="h-6" />}
			addons={
				connectionStatus !== "connected" ? (
					<ShimmerLine className="-bottom-1" />
				) : null
			}
			links={
				<NavItem asChild>
					<Link
						to="."
						search={(old) => ({ ...old, modal: "feedback" })}
					>
						Feedback
					</Link>
				</NavItem>
			}
		/>
	);
};

const Footer = () => {
	return (
		<footer className="text-muted-foreground bg-background p-4 text-center text-sm border-t relative">
			<div className="container">
				<div className="flex items-center justify-between">
					<div className="flex gap-4 items-center justify-between w-full lg:w-auto lg:justify-normal">
						<div className="flex gap-4 items-center">
							<img
								src="/logo.svg"
								alt="Rivet.gg"
								className="h-6"
							/>
							&copy; {new Date().getFullYear()}
						</div>
					</div>
					<div>{/* <CommandPanel /> */}</div>
				</div>
				<div className="flex flex-col lg:flex-row items-center justify-between mt-4 gap-4 lg:gap-0 lg:mt-8 mb-4">
					<div className="text-base flex items-center gap-4">
						<NavItem
							href="https://rivet.gg/discord"
							target="_blank"
							rel="noreferrer"
						>
							<Icon icon={faDiscord} />
						</NavItem>
						<NavItem
							href="https://github.com/rivet-gg"
							target="_blank"
							rel="noreferrer"
						>
							<Icon icon={faGithub} />
						</NavItem>
						<NavItem
							href="https://bsky.app/profile/rivet.gg"
							target="_blank"
							rel="noreferrer"
						>
							<Icon icon={faBluesky} />
						</NavItem>
						<NavItem
							href="https://x.com/rivet_gg"
							target="_blank"
							rel="noreferrer"
						>
							<Icon icon={faXTwitter} />
						</NavItem>
					</div>
					<div className="flex items-center flex-wrap justify-between lg:justify-normal w-full lg:w-auto gap-4 lg:gap-8">
						<NavItem
							href="https://rivet.gg"
							target="_blank"
							rel="noreferrer"
						>
							Home
						</NavItem>
						<NavItem
							href="https://rivet.gg/support"
							target="_blank"
							rel="noreferrer"
						>
							Help
						</NavItem>
						<NavItem
							href="https://rivet.gg/docs"
							target="_blank"
							rel="noreferrer"
						>
							Docs
						</NavItem>
					</div>
				</div>
			</div>
		</footer>
	);
};

export { Root, Main, Header, Footer, VisibleInFull };
