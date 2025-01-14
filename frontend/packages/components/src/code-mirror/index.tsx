import { forwardRef } from "react";
import ReactCodeMirror, { type ReactCodeMirrorRef, type ReactCodeMirrorProps } from "@uiw/react-codemirror";
import { githubDark, githubDarkInit } from "@uiw/codemirror-theme-github";
import { Annotation } from "@codemirror/state";
import { json, jsonParseLinter } from "@codemirror/lang-json";
import { linter } from "@codemirror/lint";
import { CodeMirrorContainer } from "../code-mirror-container";

const theme = githubDarkInit({
	settings: {
		background: "transparent",
		lineHighlight: "transparent",
		fontSize: "12px",
	},
});

export const CodeMirror = forwardRef<ReactCodeMirrorRef, ReactCodeMirrorProps>((props, ref) => {
	return <ReactCodeMirror ref={ref} theme={theme} {...props} />;
});

interface JsonCodeProps extends ReactCodeMirrorProps {}

export const JsonCode = forwardRef<HTMLDivElement, JsonCodeProps>(
	({ value, extensions = [], className, ...props }, ref) => {
		return (
			<CodeMirrorContainer ref={ref} tabIndex={0} className={className}>
				<ReactCodeMirror
					{...props}
					extensions={[json(), linter(jsonParseLinter()), ...extensions]}
					theme={githubDark}
					value={value}
				/>
			</CodeMirrorContainer>
		);
	},
);

export const External = Annotation.define<boolean>();
export { defaultKeymap } from "@codemirror/commands";
export { keymap, type KeyBinding, EditorView } from "@codemirror/view";
export { javascript, javascriptLanguage } from "@codemirror/lang-javascript";
export { type CompletionContext } from "@codemirror/autocomplete";
export { json, jsonParseLinter } from "@codemirror/lang-json";
export {
	type ReactCodeMirrorProps as CodeMirrorProps,
	type ReactCodeMirrorRef as CodeMirrorRef,
} from "@uiw/react-codemirror";
