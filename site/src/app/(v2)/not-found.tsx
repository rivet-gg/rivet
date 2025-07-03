import { Button } from "@/components/Button";
import { Icon, faBlockQuestion } from "@rivet-gg/icons";

function PageNotFound() {
	return (
		<div className="flex min-h-[80vh] w-full items-center justify-center text-center">
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
	);
}

export default PageNotFound;
