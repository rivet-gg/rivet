const path = require("node:path");
const { spawnSync } = require("node:child_process");

let __cache_packages = null;
let __cache_actorCorePackages = null;

function getPackages() {
	if(__cache_packages){
		return __cache_packages;
	}

	const out = spawnSync("pnpm", ["recursive", "list", "--json"], {cwd: __dirname});
	const packages = JSON.parse(out.stdout.toString());
	__cache_packages = packages;
	return __cache_packages;
}

function getActorCorePackages() {
	if(__cache_actorCorePackages) {
		return __cache_actorCorePackages;
	}
	const out = spawnSync("pnpm", ["recursive", "list", "--json"], {cwd: path.join(__dirname, "../actor-core")});
	if (out.status !== 0) {
		throw out.error;
	}
	const packages = JSON.parse(out.stdout.toString());
	const actorCorePackages = packages.filter((pkg) => pkg.private !== true);

	if (actorCorePackages.length === 0) {
		throw new Error("No actor-core packages found");
	}

	__cache_actorCorePackages = actorCorePackages;
	return __cache_actorCorePackages;
}

function readPackage(pkg, context) {
	if (pkg.name === "@rivet-gg/icons") {
		const output = spawnSync("pnpm", [
			"view",
			"@awesome.me/kit-63db24046b",
		]);

		if (output.status !== 0) {
			context.log(
				"Unfortunately, you are not eligible to use premium FontAwesome icons used in @rivet-gg/icons package.",
			);
			context.log("All premium icons are replaced with a square icon.");
			context.log(
				"To use all the icons, please configure your .npmrc with the following line:",
			);
			context.log("");
			context.log("@awesome.me:registry=https://npm.fontawesome.com/");
			context.log("@fortawesome:registry=https://npm.fontawesome.com/");
			context.log("//npm.fontawesome.com/:_authToken=YOUR_TOKEN");
			context.log("");
			context.log("Then, run `pnpm install` again.");
			context.log(
				"If you have a token, please check if you have access to the package.",
			);
			return pkg;
		}

		context.log(
			"You're eligible to use all icons from @rivet-gg/icons package.",
		);

		return {
			...pkg,
			dependencies: {
				"@awesome.me/kit-63db24046b": "^1.0.11",
				"@fortawesome/pro-regular-svg-icons": "6.6.0",
				"@fortawesome/pro-solid-svg-icons": "6.6.0",
			},
		};
	}

	// #region actor-core linking
	const packages = getPackages();
	const currentPackage = packages.find((p) => p.name === pkg.name);
	
	const actorCore = getActorCorePackages();
	
	for(const [dep, version] of Object.entries(pkg.dependencies)) {
		const actorCorePackage = actorCore.find((p) => p.name === dep);
		if(!actorCorePackage) {
			continue;
		}

		const relativePath = path.relative(currentPackage.path, actorCorePackage.path);

		pkg.dependencies[dep] = `file:${relativePath}`;
		context.log(
			`${pkg.name}: Linking "${dep}" to local version "${relativePath}".`,
		);
	}
	// #endregion

	return pkg;
}

function afterAllResolved(lockfile, context) {
	return lockfile
  }

module.exports = {
	hooks: {
		readPackage,
		afterAllResolved,
	},
};
