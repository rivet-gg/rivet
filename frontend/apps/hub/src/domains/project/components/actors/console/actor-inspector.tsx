import type { ComponentProps } from "react";
import { Inspector, type ObjectInspector, chromeDark } from "react-inspector";

const INSPECTOR_THEME = {
	...chromeDark,
	BASE_BACKGROUND_COLOR: "transparent",
};

export function ActorObjectInspector(
	props: ComponentProps<typeof ObjectInspector>,
) {
	return (
		<Inspector
			{...props}
			table={false}
			data={props.data}
			// Invalid types for theme
			// @ts-ignore
			theme={INSPECTOR_THEME}
		/>
	);
}
