import {
	CodeMirror,
	type CodeMirrorRef,
	type CompletionContext,
	EditorView,
	External,
	defaultKeymap,
	javascript,
	javascriptLanguage,
	keymap,
} from "@rivet-gg/components/code-mirror";
import { forwardRef } from "react";

const deleteBgTheme = EditorView.theme({
	".cm-content": { padding: 0 },
});

interface ReplInputProps {
	rpcs: string[];
	onRun: (code: string) => void;
}

export const ReplInput = forwardRef<CodeMirrorRef, ReplInputProps>(
	({ rpcs, onRun }, ref) => {
		const rivetKeymap = keymap.of([
			{
				key: "Shift-Enter",
				run: (editor) => {
					onRun(editor?.state.doc.toString());
					editor.dispatch({
						changes: {
							from: 0,
							to: editor.state.doc.length,
							insert: "",
						},
						annotations: [External.of(true)],
					});
					return true;
				},
			},
			...defaultKeymap,
		]);

		const replAutocomplete = javascriptLanguage.data.of({
			autocomplete: (context: CompletionContext) => {
				const word = context.matchBefore(/^\w*/);
				if (!word || (word?.from === word?.to && !context.explicit))
					return null;
				return {
					from: word.from,
					to: word.to,
					boost: 99,
					options: [
						{
							label: "wait",
							apply: "wait",
							validFor: /^(@\w*)?$/,
							info: "Helper function to wait for a number of milliseconds",
						},
						...rpcs.map((rpc) => ({
							label: rpc,
							apply: rpc,
							validFor: /^(@\w*)?$/,
							info: `Call "${rpc}" RPC on Actor`,
						})),
					],
				};
			},
		});

		return (
			<CodeMirror
				autoFocus
				basicSetup={{
					lineNumbers: false,
					lintKeymap: false,
					foldKeymap: false,
					searchKeymap: false,
					defaultKeymap: false,
					foldGutter: false,
				}}
				extensions={[
					deleteBgTheme,
					rivetKeymap,
					javascript(),
					replAutocomplete,
				]}
			/>
		);
	},
);
