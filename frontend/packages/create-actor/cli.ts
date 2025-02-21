import assert from "node:assert";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import k from "kleur";
import prompts from "prompts";
import { version } from "./package.json";

const TEMPLATES = [
	{
		title: "Counter (TypeScript)",
		value: "counter",
	},
	{ title: "Blank (TypeScript)", value: "blank-ts" },
	{ title: "Blank (JavaScript)", value: "blank-js" },
];

const PLATFORMS = [
	{ title: "Rivet", value: "rivet" },
	{ title: "Cloudflare Workers", value: "cloudflare" },
	{ title: "Node", value: "node" },
	{ title: "Bun", value: "bun" },
];

const RENAME_FILES: Record<string, string> = {
	_gitignore: ".gitignore",
};

// MARK: Helper functions
function isEmpty(path: string) {
	const files = fs.readdirSync(path);
	return files.length === 0 || (files.length === 1 && files[0] === ".git");
}

function isSafeToWrite(path: string) {
	return !fs.existsSync(path) || isEmpty(path);
}

function copy(src: string, dest: string) {
	const stat = fs.statSync(src);
	if (stat.isDirectory()) {
		copyDir(src, dest);
	} else {
		fs.copyFileSync(src, dest);
	}
}

function copyDir(srcDir: string, destDir: string) {
	fs.mkdirSync(destDir, { recursive: true });
	for (const file of fs.readdirSync(srcDir)) {
		const srcFile = path.resolve(srcDir, file);
		const destFile = path.resolve(destDir, file);
		copy(srcFile, destFile);
	}
}

function pkgFromUserAgent(userAgent: string | undefined):
	| {
			name: string;
			version: string;
	  }
	| undefined {
	if (!userAgent) return undefined;
	const pkgSpec = userAgent.split(" ")[0];
	const pkgSpecArr = pkgSpec.split("/");
	return {
		name: pkgSpecArr[0],
		version: pkgSpecArr[1],
	};
}

function write(root: string, file: string, content?: string) {
	const targetPath = path.join(root, RENAME_FILES[file] ?? file);
	if (content) {
		fs.writeFileSync(targetPath, content);
	} else {
		copy(path.join(templateDir, file), targetPath);
	}
}

function emptyDir(dir: string) {
	if (!fs.existsSync(dir)) {
		return;
	}
	for (const file of fs.readdirSync(dir)) {
		if (file === ".git") {
			continue;
		}
		fs.rmSync(path.resolve(dir, file), { recursive: true, force: true });
	}
}

// MARK: Parse arguments
const { values, positionals } = parseArgs({
	args: process.argv.slice(2),
	options: {
		help: {
			type: "boolean",
			short: "h",
		},
		template: {
			type: "string",
			short: "t",
		},
		overwrite: {
			type: "boolean",
			short: "o",
		},
		platform: {
			type: "string",
			short: "p",
		},
	},
	allowPositionals: true,
});

console.log(`🎭 ${k.red().bold("Create Actor")} ${k.gray(`v${version}`)}`);
console.log();

prompts.override({
	overwrite: values.overwrite,
});

let { template, platform, overwrite } = values;
let [folderName] = positionals;

// MARK: Target folder question
if (!folderName) {
	const response = await prompts(
		{
			type: "text",
			name: "name",
			message: "Enter a folder name",
			validate: (value) => (value ? true : "Folder name cannot be empty"),
		},
		{ onCancel: () => process.exit(1) },
	);
	folderName = response.name;
}

const targetDir = path.join(process.cwd(), folderName);

if (!isSafeToWrite(targetDir)) {
	console.log(
		k.red(
			`✖ Specified directory ${k.underline(
				`${targetDir}`,
			)} is not empty. Please choose an empty directory or use --overwrite flag.`,
		),
	);
	process.exit(1);
}

// MARK: Template question
if (!template) {
	const response = await prompts(
		{
			type: "select",
			name: "template",
			message: "Choose template",
			choices: TEMPLATES,
		},
		{ onCancel: () => process.exit(1) },
	);
	template = response.template;
}
assert(template !== undefined, "Template must be defined");

// MARK: Platform question
if (!platform) {
	const response = await prompts(
		{
			type: "select",
			name: "platform",
			message: "Choose platform",
			choices: PLATFORMS,
		},
		{ onCancel: () => process.exit(1) },
	);
	platform = response.platform;
}
assert(platform !== undefined, "Platform must be defined");

// MARK: Copy template files
console.log(`🔨 Creating new actor in ${k.underline(targetDir)}...`);
const templateDir = path.join(__dirname, `template-${template}`);
const files = fs.readdirSync(templateDir, {
	recursive: true,
	encoding: "utf-8",
});

if (overwrite) {
	emptyDir(targetDir);
} else if (!fs.existsSync(targetDir)) {
	fs.mkdirSync(targetDir, { recursive: true });
}

for (const file of files.filter((f) => f !== "package.json")) {
	write(targetDir, file);
}
const pkg = JSON.parse(
	fs.readFileSync(path.join(templateDir, "package.json"), "utf-8"),
);

pkg.name = path.basename(path.resolve(targetDir));
pkg.dependencies["@rivet-gg/actor"] = version;
pkg.devDependencies["@rivet-gg/actor-client"] = version;

write(targetDir, "package.json", `${JSON.stringify(pkg, null, 2)}\n`);

// MARK: Run instructions
const pkgInfo = pkgFromUserAgent(process.env.npm_config_user_agent);
const pkgManager = pkgInfo?.name || "npm";
const runDevCmd = pkgManager === "yarn" ? "yarn dev" : `${pkgManager} run dev`;

console.log(`
✨ Done. To get started:

   cd ${folderName}
   ${pkgManager} install
   ${runDevCmd}
    
Happy hacking! 🚀
`);
