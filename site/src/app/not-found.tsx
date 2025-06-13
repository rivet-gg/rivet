import { Button } from "@/components/Button";
import { Footer } from "@/components/Footer";
import { Icon, faBlockQuestion } from "@rivet-gg/icons";
import "@/styles/tailwind.css";
import "@/styles/fonts.css";
import { FancyHeader } from "@/components/v2/FancyHeader";

function PageNotFound() {
	return (
		<>
			<FancyHeader />
			<div className="relative pt-14">
				<div className="relative flex min-h-[80vh] w-full items-center justify-center text-center">
					<div className="transition-opacity">
						<h1 className="mb-3 flex items-center justify-center text-3xl text-white">
							<Icon
								className="mr-2 size-10"
								icon={faBlockQuestion}
							/>{" "}
							Not Found
						</h1>
						<Button href="/" variant="secondary">
							Home
						</Button>
					</div>
				</div>
			</div>
			<Footer />
		</>
	);
}

export default PageNotFound;
