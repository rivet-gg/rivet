import GameClient from "./state";

const SHORT_SIDE_DIMENSION_GAME_UNITS = 1000;
const IS_SAFARI = navigator.userAgent.includes("Safari") && !navigator.userAgent.includes("Chrome");
const IS_CHROME = navigator.userAgent.includes("Chrome");
const IS_FIREFOX = navigator.userAgent.includes("Firefox");

export function getTextScale(screenScale: number): number {
    return getPixelScalar() / screenScale;
}

function getZoomValue(): number {
    return (window.outerWidth - 10) / window.innerWidth;
}

export function getPixelScalar(): number {
    if (IS_SAFARI) return getZoomValue() * window.devicePixelRatio;
    else return window.devicePixelRatio;
}

function getRenderingDimensions() {
    const rawDimensions = { w: window.innerWidth, h: window.innerHeight };

    return { w: rawDimensions.w * getPixelScalar(), h: rawDimensions.h * getPixelScalar() };
}

function getNormalizedDimensions() {
    const rawDimensions = { w: window.innerWidth, h: window.innerHeight };
    const minDimension = Math.min(rawDimensions.w, rawDimensions.h);
    const rawScalar = SHORT_SIDE_DIMENSION_GAME_UNITS / minDimension;

    return { w: rawDimensions.w * rawScalar, h: rawDimensions.h * rawScalar };
}

export function resizeClient(client: GameClient) {
    const renderingSizes = getRenderingDimensions();
    const apparentSizes = getNormalizedDimensions();

    client.canvas.width = renderingSizes.w;
    client.canvas.height = renderingSizes.h;

    client.screenSize.w = apparentSizes.w;
    client.screenSize.h = apparentSizes.h;

    client.screenScale = renderingSizes.w / apparentSizes.w;
}
