import type { Metadata } from "next";
import Link from "next/link";
import { Button } from "@rivet-gg/components";
import {
	Icon,
	faCode,
	faLayerGroup,
	faTerminal,
	faDesktop,
	faListCheck,
	faArrowsToCircle,
	faReact,
	faVuejs,
	faAngular,
	faNodeJs,
	faPython,
	faPhp,
	faJava,
	faCss3Alt,
	faHtml5,
	faRust,
	faSwift,
	faJsSquare,
	faGolang,
	faDatabase,
	faDocker,
	faArrowRight,
	faRobot,
	faServer,
	faVectorSquare,
} from "@rivet-gg/icons";
import { CopyCommand } from "./CopyCommand";
import { TutorialsSection } from "./TutorialsSection";
import { CommandCenterSection } from "./CommandCenterSection";
import { CommunitySection } from "./CommunitySection";
import { CtaSection } from "./CtaSection";
import { MarketingButton } from "./MarketingButton";
import GlobeSvg from "../(img)/globe.svg";
import ActorsSvg from "../(img)/actors.svg";
import ContainerSvg from "../(img)/container.svg";
import WorkflowSvg from "../(img)/workflow.svg";
import DbSvg from "../(img)/db.svg";
import Image from "next/image";

export const metadata: Metadata = {
	title: "Rivet - The Open-Source Serverless Platform",
	description:
		"Easily build & scale AI agents, functions, and more. Open-source & self-hostable.",
};

export default function IndexPage() {
	// an empty div at the top of the page is a workaround for a bug in Next.js that causes the page to jump when the user navigates to it
	// https://github.com/vercel/next.js/discussions/64534
	return (
		<>
			<div />
			<main className="min-h-screen w-full max-w-[1500px] mx-auto md:px-8">
				<FeaturesGrid />
				<Hero />
				<FrameworksSection />
				<TutorialsSection />
				<CommandCenterSection />
				<CommunitySection />
				<CtaSection />
			</main>
		</>
	);
}
// Hero component with title, subtitle, and CTA buttons
const Hero = () => {
	return (
		<div className="relative isolate overflow-hidden pb-8 sm:pb-10">
			<div className="mx-auto max-w-[1200px] px-6 lg:px-8">
				<div className="max-w-2xl ml-auto text-right">
					{/* Title */}
					<div className="space-y-6">
						<h1 className="text-6xl font-700 text-white leading-[1.1] tracking-normal">
							The open-source
							<br className="hidden sm:inline" />
							serverless platform
						</h1>
						<p className="text-xl leading-[1.2] tracking-tight font-500 text-white/60 max-w-lg ml-auto">
							Easily deploy & scale{" "}
							<span className="text-white/80">AI agents</span>,{" "}
							<span className="text-white/80">
								complex workloads
							</span>,{" "}
							<span className="text-white/80">backends</span>{" "}
							— all on a frictionless platform that runs anywhere.
						</p>
					</div>

					<div className="h-10" />

					{/* CTA */}
					<div className="flex flex-col sm:flex-row items-end gap-4 justify-end">
						<MarketingButton href="#deploy" primary>
							Deploy Now
						</MarketingButton>
						<MarketingButton href="/rivet-vs-cloudflare-workers">
							<span>On-Prem Cloudflare Workers</span>
							<Icon
								icon={faArrowRight}
								className="ml-2 text-xs group-hover:translate-x-0.5 transition-transform"
							/>
						</MarketingButton>
					</div>
				</div>
			</div>
		</div>
	);
};

// Feature component for individual features
const Feature = ({
	title,
	description,
	faIcon,
	href,
	useCases,
}: {
	title: string;
	description: string;
	faIcon: any;
	href: string;
	useCases?: string[];
}) => {
	return (
		<Link href={href} className="block group">
			<div className="rounded-xl bg-[#FF5C00]/90 group-hover:bg-[#FF5C00] group-hover:brightness-110 border border-[#FF5C00]/20 group-hover:border-[#FF5C00]/30 shadow-sm transition-all duration-200 relative overflow-hidden h-[450px] w-[360px] flex flex-col">
				<div className="px-8 mt-6">
					<div className="flex items-center gap-3 mb-6">
						<Icon
							icon={faIcon}
							className="text-lg text-black/90 group-hover:text-black transition-colors duration-300"
						/>
						<h3 className="text-lg font-normal text-black/70 group-hover:text-black/90 transition-colors duration-200">
							{title}
						</h3>
					</div>

					<p className="text-black/60 group-hover:text-black/80 transition-colors duration-200 text-sm">
						{description}
					</p>

					{useCases && useCases.length > 0 && (
						<div className="mt-3 flex flex-wrap gap-x-2 text-xs">
							<span className="text-black/50 group-hover:text-black/70 transition-colors duration-200">Good for:</span>
							{useCases.map((useCase, index) => (
								<span key={index} className="text-black/60 group-hover:text-black/80 transition-colors duration-200">
									{useCase}
									{index < useCases.length - 1 ? "," : ""}
								</span>
							))}
						</div>
					)}
				</div>

				<div className="mt-auto">
					{title === "Stateless Functions" && (
						<div className="absolute bottom-0 left-0 h-80 w-80 opacity-10 group-hover:opacity-15 transition-opacity duration-200 -ml-8 -mb-36">
							<Image
								src={GlobeSvg}
								alt="Globe"
								fill
								className="object-contain scale-105 brightness-0"
							/>
						</div>
					)}
					{title === "Stateful Actors" && (
						<div className="absolute top-[240px] left-0 h-64 w-64 opacity-10 group-hover:opacity-15 transition-opacity duration-200 -ml-8">
							<Image
								src={ActorsSvg}
								alt="Actors"
								fill
								className="object-contain scale-105 brightness-0"
							/>
						</div>
					)}
					{title === "Sandboxed Containers" && (
						<div className="absolute top-[240px] left-0 h-80 w-80 opacity-10 group-hover:opacity-15 transition-opacity duration-200 -ml-[100px]">
							<Image
								src={ContainerSvg}
								alt="Container"
								fill
								className="object-contain scale-105 brightness-0"
							/>
						</div>
					)}
					{title === "Workflows" && (
						<div className="absolute top-[200px] left-0 h-80 w-80 opacity-10 group-hover:opacity-15 transition-opacity duration-200 -ml-[100px]">
							<Image
								src={WorkflowSvg}
								alt="Workflow"
								fill
								className="object-contain scale-105 brightness-0"
							/>
						</div>
					)}
					{title === "SQLite Databases" && (
						<div className="absolute top-[200px] left-0 h-80 w-80 opacity-10 group-hover:opacity-15 transition-opacity duration-200 -ml-[100px]">
							<Image
								src={DbSvg}
								alt="Database"
								fill
								className="object-contain scale-105 brightness-0"
							/>
						</div>
					)}
					<div className="px-8 pb-8 relative z-10">
						<div className="flex items-center justify-end text-black opacity-0 group-hover:opacity-100 transition-opacity">
							<Icon
								icon={faArrowRight}
								className="text-xl group-hover:translate-x-0.5 transition-all"
							/>
						</div>
					</div>
				</div>
			</div>
		</Link>
	);
};

// Features grid component
const FeaturesGrid = () => {
	const features = [
		{
			title: "Stateless Functions",
			description: "Deploy serverless functions that scale automatically",
			faIcon: faCode,
			href: "/docs/functions",
			useCases: ["APIs", "Edge computing", "Microservices"],
		},
		{
			title: "Stateful Actors",
			description:
				"Long running tasks with state persistence, hibernation, and realtime",
			faIcon: faLayerGroup,
			href: "/docs/stateful-jobs",
			useCases: ["AI agents", "Realtime apps", "Local-first sync"],
		},
		{
			title: "Sandboxed Containers",
			description:
				"Run CPU- & memory-intensive workloads in secure containers with blazing fast coldstarts",
			faIcon: faServer,
			href: "/docs/stateful-jobs",
			useCases: ["Code interpreters", "Remote desktop", "Game servers"],
		},
		//{
		//	title: "Workflows",
		//	description: "Orchestrate complex, multi-step processes",
		//	faIcon: faArrowsToCircle,
		//	href: "/docs/workflows",
		//	useCases: ["AI agents", "Business logic", "Data pipelines"]
		//},
		//{
		//	title: "SQLite Databases",
		//	description: "On-demand SQL databases 10x faster than Postgres with vector stores & full text search",
		//	faIcon: faDatabase,
		//	href: "/docs/sqlite-databases",
		//	useCases: ["Agent memory", "Per-tenant databases", "Local-first apps"]
		//},
	];

	return (
		<div className="mx-auto w-full px-4 pt-32 pb-16 max-w-[1200px]">
			<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 justify-items-center">
				{features.map((feature, index) => (
					<Feature
						key={index}
						title={feature.title}
						description={feature.description}
						faIcon={feature.faIcon}
						href={feature.href}
						useCases={feature.useCases}
					/>
				))}
			</div>
		</div>
	);
};

// Frameworks section
const FrameworksSection = () => {
	const frameworks = [
		{ icon: faReact, name: "React", href: "/docs/frameworks/react" },
		{ icon: faVuejs, name: "Vue", href: "/docs/frameworks/vue" },
		{ icon: faAngular, name: "Angular", href: "/docs/frameworks/angular" },
		{ icon: faNodeJs, name: "Node.js", href: "/docs/frameworks/nodejs" },
		{ icon: faPython, name: "Python", href: "/docs/frameworks/python" },
		{ icon: faPhp, name: "PHP", href: "/docs/frameworks/php" },
		{ icon: faJava, name: "Java", href: "/docs/frameworks/java" },
		{ icon: faRust, name: "Rust", href: "/docs/frameworks/rust" },
		{ icon: faSwift, name: "Swift", href: "/docs/frameworks/swift" },
		{
			icon: faJsSquare,
			name: "JavaScript",
			href: "/docs/frameworks/javascript",
		},
		{ icon: faHtml5, name: "HTML5", href: "/docs/frameworks/html5" },
		{ icon: faCss3Alt, name: "CSS3", href: "/docs/frameworks/css3" },
		{ icon: faGolang, name: "Go", href: "/docs/frameworks/go" },
		{ icon: faDatabase, name: "SQL", href: "/docs/frameworks/sql" },
		{ icon: faDocker, name: "Docker", href: "/docs/frameworks/docker" },
	];

	return (
		<div className="mx-auto max-w-7xl px-6 py-28 lg:py-44 lg:px-8 mt-16">
			<div className="flex flex-col md:flex-row md:items-start">
				<div className="grow max-w-lg mb-8 md:mb-0 md:pr-8">
					<h2 className="text-4xl font-medium tracking-tight text-white text-left">
						Rivet works with any framework
					</h2>
					<p className="mt-4 text-lg text-white/70 text-left">
						Integrate with your existing tech stack or start fresh
						with your preferred tools and languages.
					</p>
				</div>
				<div>
					<div className="grid grid-cols-4 gap-x-6 gap-y-6">
						{frameworks.map((framework, index) => (
							<Link
								key={index}
								href={framework.href}
								className="group"
							>
								<div className="h-16 w-16 mx-auto flex items-center justify-center">
									<Icon
										icon={framework.icon}
										className="text-5xl text-white/30 group-hover:text-white/90 transition-colors duration-200"
										title={framework.name}
									/>
								</div>
							</Link>
						))}
					</div>
				</div>
			</div>
		</div>
	);
};
