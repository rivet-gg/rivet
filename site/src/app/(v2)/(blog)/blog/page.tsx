import { loadArticles } from "@/lib/article";
import { formatTimestamp } from "@/lib/formatDate";
import {
	Avatar,
	AvatarFallback,
	AvatarImage,
	Badge,
	Button,
} from "@rivet-gg/components";
import { Icon, faRss } from "@rivet-gg/icons";
import type { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import type { CSSProperties } from "react";

export const metadata: Metadata = {
	title: "Blog - Rivet",
};

function Article({
	title,
	description,
	slug,
	published,
	author,
	image,
	category,
}) {
	const href = category.id === "changelog" ? `/changelog/${slug}` : `/blog/${slug}`;
	return (
		<Link href={href} className="size-full">
			<article className="bg-card hover:border-primary flex size-full flex-col items-start justify-between rounded-md border p-4 transition-colors">
				<div>
					{/* Image */}
					<div className="relative w-full">
						<Image
							src={image}
							alt={"hero"}
							className="aspect-[2/1] w-full rounded-md border object-cover"
						/>
					</div>

					{/* Date & category */}
					<div className="mt-3 flex items-center gap-x-3 text-xs">
						<time
							dateTime={formatTimestamp(published)}
							className="text-muted-foreground"
						>
							{formatTimestamp(published)}
						</time>
						<Badge variant="outline">{category.name}</Badge>
					</div>

					{/* Description */}
					<div className="group relative">
						<h3 className="mt-2 text-lg font-bold leading-6">
							{title}
						</h3>
						<p className="text-muted-foreground mt-3 line-clamp-3 text-sm leading-6">
							{description}
						</p>
					</div>
				</div>

				<div className="max-w-xl">
					{/* Author */}
					<div className="relative mt-4 flex items-center gap-x-4">
						<Avatar>
							<AvatarFallback>{author[0]}</AvatarFallback>
							<AvatarImage {...author.avatar} alt={author.name} />
						</Avatar>
						<div className="text-sm">
							<div className="font-semibold">{author.name}</div>
						</div>
					</div>
				</div>
			</article>
		</Link>
	);
}

export default async function BlogPage() {
	const articles = await loadArticles();

	const posts = articles.filter(
		(article) => article.category.id !== "changelog",
	);

	return (
		<div
			className="mx-auto mt-20 w-full max-w-6xl px-8 md:mt-32"
			style={{ "--header-height": "5rem" } as CSSProperties}
		>
			<div className="mt-8 flex w-full items-center justify-between">
				<h1 className="text-6xl font-bold">Blog</h1>
				<Button asChild startIcon={<Icon icon={faRss} />}>
					<Link href="/rss/feed.xml">RSS Feed</Link>
				</Button>
			</div>
			<div className="mb-8 mt-4 flex items-center justify-start">
				<div className="bg-card rounded-md border">
					<Button variant="secondary" asChild>
						<Link href="/blog">All Posts</Link>
					</Button>
					<Button variant="ghost" asChild>
						<Link href="/changelog">Changelog</Link>
					</Button>
				</div>
			</div>
			<div className="mb-8 mt-8 grid grid-cols-1 gap-8 md:grid-cols-3">
				{posts
					.sort((a, b) => b.published - a.published)
					.map((article) => (
						<Article key={article.slug} {...article} />
					))}
			</div>
		</div>
	);
}
