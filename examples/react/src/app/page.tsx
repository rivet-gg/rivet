"use client";

import { Client } from "@rivet-gg/actor-client";
import { ActorClientProvider } from "@rivet-gg/actor-client/unstable-react";
import { Suspense } from "react";
import styles from "./page.module.css";
import { ServerDrivenUi } from "./server-driven-ui";
import { SimpleChat } from "./simple-chat";

// biome-ignore lint/style/noNonNullAssertion: it's fine
const actorClient = new Client(process.env.NEXT_PUBLIC_ACTOR_MANAGER_URL!);

export default function Home() {
	return (
		<div className={styles.page}>
			<main className={styles.main}>
				<ActorClientProvider client={actorClient}>
					<div>
						<img
							src="/logo.svg"
							alt="Rivet Logo"
							className={styles.logo}
						/>
						<h1>Rivet Actors &times; React</h1>
					</div>

					<section>
						<p>
							This chat is powered by Rivet Actors and React. Open
							a new tab to see changes in real-time. Send messages
							and receive them.
						</p>
						<Suspense>
							<SimpleChat />
						</Suspense>
					</section>
					<section>
						<p>
							Lorem ipsum dolor sit amet, consectetur adipiscing
							elit. Nullam nec purus
						</p>
						<ServerDrivenUi />
					</section>
				</ActorClientProvider>
			</main>
		</div>
	);
}
