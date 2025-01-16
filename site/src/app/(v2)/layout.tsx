import { Footer } from "@/components/Footer";
import { EmbedDetector } from "@/components/EmbedDetector";
import "@/styles/v2.css";

export default function Layout({ children }) {
	return (
		<>
			{children}
			<EmbedDetector />
			<Footer />
		</>
	);
}
