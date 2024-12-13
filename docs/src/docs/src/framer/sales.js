// @ts-nocheck
/* eslint-disable */
'use client';
import { className, className2, css, css2, fonts, fonts2, } from './chunk-76DSJYNX.js';
import './chunk-V5TDMFQ4.js';

// https :https://framerusercontent.com/modules/ow6h1b6wQLBemft55pFs/fiMzU6TRqHLGNmiUHMjg/XndMdburz.js
import { jsx as _jsx4, jsxs as _jsxs3, } from 'react/jsx-runtime';
import {
  addFonts as addFonts3,
  addPropertyControls as addPropertyControls3,
  ComponentViewportProvider as ComponentViewportProvider2,
  ControlType as ControlType3,
  cx as cx3,
  getFonts as getFonts2,
  getFontsFromSharedStyle,
  RichText as RichText3,
  useComponentViewport as useComponentViewport3,
  useLocaleInfo as useLocaleInfo3,
  useVariantState as useVariantState3,
  withCSS as withCSS3,
} from 'unframer';
import { LayoutGroup as LayoutGroup3, motion as motion3, MotionConfigContext as MotionConfigContext3, } from 'unframer';
import * as React3 from 'react';

// https :https://framerusercontent.com/modules/otCMm3mRTsMjv8dlT5SK/asZ5tK2IzCQkKFFatnw6/PosthogForm.js
import { jsx as _jsx, } from 'react/jsx-runtime';
import { addPropertyControls, ControlType, } from 'unframer';
function PostHogForm({ children, ...props },) {
  return /* @__PURE__ */ _jsx('form', { ...props, children, },);
}
addPropertyControls(PostHogForm, { children: { type: ControlType.ComponentInstance, }, onSubmit: { type: ControlType.EventHandler, }, },);

// https :https://framerusercontent.com/modules/LNe7c2MkdJc10yncD9lr/PDFJYR9t4CoWEFQVpcy5/WeRAxkTPv.js
import { jsx as _jsx3, jsxs as _jsxs2, } from 'react/jsx-runtime';
import {
  addFonts as addFonts2,
  ComponentViewportProvider,
  cx as cx2,
  FormPlainTextInput,
  getFonts,
  RichText as RichText2,
  useActiveVariantCallback as useActiveVariantCallback2,
  useComponentViewport as useComponentViewport2,
  useLocaleInfo as useLocaleInfo2,
  useVariantState as useVariantState2,
  withCSS as withCSS2,
} from 'unframer';
import { LayoutGroup as LayoutGroup2, motion as motion2, MotionConfigContext as MotionConfigContext2, } from 'unframer';
import * as React2 from 'react';

// https :https://framerusercontent.com/modules/91AyXErLKVPpIjW2QRmt/hPIJGIPfIB3tNpuMfWr9/JGT1yLjbN.js
import { jsx as _jsx2, jsxs as _jsxs, } from 'react/jsx-runtime';
import {
  addFonts,
  addPropertyControls as addPropertyControls2,
  ControlType as ControlType2,
  cx,
  RichText,
  useActiveVariantCallback,
  useComponentViewport,
  useLocaleInfo,
  useVariantState,
  withCSS,
  withFX,
} from 'unframer';
import { LayoutGroup, motion, MotionConfigContext, } from 'unframer';
import * as React from 'react';
var MotionDivWithFX = withFX(motion.div,);
var enabledGestures = { Sc0Dg6vVo: { hover: true, pressed: true, }, };
var cycleOrder = ['Sc0Dg6vVo', 'eY3CsaRlv', 'Vt9zLruWe', 'dfgg9_FTh', 'MolWXut_w',];
var serializationHash = 'framer-Rde70';
var variantClassNames = {
  dfgg9_FTh: 'framer-v-3zzmgb',
  eY3CsaRlv: 'framer-v-sqy5j3',
  MolWXut_w: 'framer-v-muutrj',
  Sc0Dg6vVo: 'framer-v-1v01vfs',
  Vt9zLruWe: 'framer-v-1r2hzz9',
};
function addPropertyOverrides(overrides, ...variants) {
  const nextOverrides = {};
  variants === null || variants === void 0
    ? void 0
    : variants.forEach((variant,) => variant && Object.assign(nextOverrides, overrides[variant],));
  return nextOverrides;
}
var transition1 = { delay: 0, duration: 0.2, ease: [0.44, 0, 0.56, 1,], type: 'tween', };
var transition2 = { delay: 0, duration: 1, ease: [0, 0, 1, 1,], type: 'tween', };
var animation = { opacity: 1, rotate: 360, rotateX: 0, rotateY: 0, scale: 1, skewX: 0, skewY: 0, x: 0, y: 0, };
var transformTemplate1 = (_, t,) => `translateX(-50%) ${t}`;
var Transition = ({ value, children, },) => {
  const config = React.useContext(MotionConfigContext,);
  const transition = value !== null && value !== void 0 ? value : config.transition;
  const contextValue = React.useMemo(() => ({ ...config, transition, }), [JSON.stringify(transition,),],);
  return /* @__PURE__ */ _jsx2(MotionConfigContext.Provider, { value: contextValue, children, },);
};
var Variants = motion.create(React.Fragment,);
var humanReadableVariantMap = {
  Default: 'Sc0Dg6vVo',
  Disabled: 'Vt9zLruWe',
  Error: 'MolWXut_w',
  Loading: 'eY3CsaRlv',
  Success: 'dfgg9_FTh',
};
var getProps = ({ click, height, id, width, ...props },) => {
  var _humanReadableVariantMap_props_variant, _ref;
  return {
    ...props,
    Shx5vBWP_: click !== null && click !== void 0 ? click : props.Shx5vBWP_,
    variant:
      (_ref =
            (_humanReadableVariantMap_props_variant = humanReadableVariantMap[props.variant]) !== null &&
              _humanReadableVariantMap_props_variant !== void 0
              ? _humanReadableVariantMap_props_variant
              : props.variant) !== null && _ref !== void 0
        ? _ref
        : 'Sc0Dg6vVo',
  };
};
var createLayoutDependency = (props, variants,) => {
  if (props.layoutDependency) return variants.join('-',) + props.layoutDependency;
  return variants.join('-',);
};
var Component = /* @__PURE__ */ React.forwardRef(function (props, ref,) {
  const { activeLocale, setLocale, } = useLocaleInfo();
  const { style, className: className3, layoutId, variant, Shx5vBWP_, ...restProps } = getProps(props,);
  const {
    baseVariant,
    classNames,
    clearLoadingGesture,
    gestureHandlers,
    gestureVariant,
    isLoading,
    setGestureState,
    setVariant,
    variants,
  } = useVariantState({ cycleOrder, defaultVariant: 'Sc0Dg6vVo', enabledGestures, variant, variantClassNames, },);
  const layoutDependency = createLayoutDependency(props, variants,);
  const { activeVariantCallback, delay, } = useActiveVariantCallback(baseVariant,);
  const onTap1qcrh61 = activeVariantCallback(async (...args) => {
    setGestureState({ isPressed: false, },);
    if (Shx5vBWP_) {
      const res = await Shx5vBWP_(...args,);
      if (res === false) return false;
    }
  },);
  const ref1 = React.useRef(null,);
  const isDisplayed = () => {
    if (baseVariant === 'eY3CsaRlv') return false;
    return true;
  };
  const isDisplayed1 = () => {
    if (baseVariant === 'eY3CsaRlv') return true;
    return false;
  };
  const defaultLayoutId = React.useId();
  const sharedStyleClassNames = [];
  const componentViewport = useComponentViewport();
  return /* @__PURE__ */ _jsx2(LayoutGroup, {
    id: layoutId !== null && layoutId !== void 0 ? layoutId : defaultLayoutId,
    children: /* @__PURE__ */ _jsx2(Variants, {
      animate: variants,
      initial: false,
      children: /* @__PURE__ */ _jsx2(Transition, {
        value: transition1,
        children: /* @__PURE__ */ _jsxs(motion.button, {
          ...restProps,
          ...gestureHandlers,
          className: cx(serializationHash, ...sharedStyleClassNames, 'framer-1v01vfs', className3, classNames,),
          'data-framer-name': 'Default',
          'data-highlight': true,
          'data-reset': 'button',
          layoutDependency,
          layoutId: 'Sc0Dg6vVo',
          onTap: onTap1qcrh61,
          ref: ref !== null && ref !== void 0 ? ref : ref1,
          style: {
            backgroundColor: 'rgb(255, 79, 0)',
            borderBottomLeftRadius: 10,
            borderBottomRightRadius: 10,
            borderTopLeftRadius: 10,
            borderTopRightRadius: 10,
            opacity: 1,
            ...style,
          },
          variants: {
            'Sc0Dg6vVo-hover': { backgroundColor: 'rgba(255, 81, 0, 0.63)', },
            'Sc0Dg6vVo-pressed': { backgroundColor: 'rgba(255, 81, 0, 0.47)', },
            MolWXut_w: { backgroundColor: 'rgba(255, 34, 68, 0.15)', },
            Vt9zLruWe: { opacity: 0.5, },
          },
          ...addPropertyOverrides(
            {
              'Sc0Dg6vVo-hover': { 'data-framer-name': void 0, },
              'Sc0Dg6vVo-pressed': { 'data-framer-name': void 0, },
              dfgg9_FTh: { 'data-framer-name': 'Success', },
              eY3CsaRlv: { 'data-framer-name': 'Loading', },
              MolWXut_w: { 'data-framer-name': 'Error', },
              Vt9zLruWe: { 'data-framer-name': 'Disabled', },
            },
            baseVariant,
            gestureVariant,
          ),
          children: [
            isDisplayed() && /* @__PURE__ */ _jsx2(RichText, {
              __fromCanvasComponent: true,
              children: /* @__PURE__ */ _jsx2(React.Fragment, {
                children: /* @__PURE__ */ _jsx2(motion.p, {
                  style: {
                    '--font-selector': 'SW50ZXItU2VtaUJvbGQ=',
                    '--framer-font-family': '"Inter", "Inter Placeholder", sans-serif',
                    '--framer-font-size': '14px',
                    '--framer-font-weight': '600',
                    '--framer-text-color': 'var(--extracted-r6o4lv, rgb(255, 255, 255))',
                  },
                  children: 'Submit',
                },),
              },),
              className: 'framer-bagq5',
              fonts: ['Inter-SemiBold',],
              layoutDependency,
              layoutId: 'C3iA2_u4M',
              style: {
                '--extracted-r6o4lv': 'rgb(255, 255, 255)',
                '--framer-link-text-color': 'rgb(0, 153, 255)',
                '--framer-link-text-decoration': 'underline',
              },
              variants: { MolWXut_w: { '--extracted-r6o4lv': 'rgb(255, 34, 68)', }, },
              verticalAlignment: 'top',
              withExternalLayout: true,
              ...addPropertyOverrides(
                {
                  dfgg9_FTh: {
                    children: /* @__PURE__ */ _jsx2(React.Fragment, {
                      children: /* @__PURE__ */ _jsx2(motion.p, {
                        style: {
                          '--font-selector': 'SW50ZXItU2VtaUJvbGQ=',
                          '--framer-font-family': '"Inter", "Inter Placeholder", sans-serif',
                          '--framer-font-size': '14px',
                          '--framer-font-weight': '600',
                          '--framer-text-color': 'var(--extracted-r6o4lv, rgb(255, 255, 255))',
                        },
                        children: 'Thank you',
                      },),
                    },),
                  },
                  MolWXut_w: {
                    children: /* @__PURE__ */ _jsx2(React.Fragment, {
                      children: /* @__PURE__ */ _jsx2(motion.p, {
                        style: {
                          '--font-selector': 'SW50ZXItU2VtaUJvbGQ=',
                          '--framer-font-family': '"Inter", "Inter Placeholder", sans-serif',
                          '--framer-font-size': '14px',
                          '--framer-font-weight': '600',
                          '--framer-text-color': 'var(--extracted-r6o4lv, rgb(255, 34, 68))',
                        },
                        children: 'Something went wrong',
                      },),
                    },),
                  },
                },
                baseVariant,
                gestureVariant,
              ),
            },),
            isDisplayed1() && /* @__PURE__ */ _jsx2(motion.div, {
              className: 'framer-j7pn8o',
              'data-framer-name': 'Spinner',
              layoutDependency,
              layoutId: 'himpczHBK',
              style: {
                mask: 'url(\'https://framerusercontent.com/images/pGiXYozQ3mE4cilNOItfe2L2fUA.svg\') alpha no-repeat center / cover add',
                WebkitMask:
                  'url(\'https://framerusercontent.com/images/pGiXYozQ3mE4cilNOItfe2L2fUA.svg\') alpha no-repeat center / cover add',
              },
              children: /* @__PURE__ */ _jsx2(MotionDivWithFX, {
                __framer__loop: animation,
                __framer__loopEffectEnabled: true,
                __framer__loopRepeatDelay: 0,
                __framer__loopRepeatType: 'loop',
                __framer__loopTransition: transition2,
                __perspectiveFX: false,
                __smartComponentFX: true,
                __targetOpacity: 1,
                className: 'framer-11aixt1',
                'data-framer-name': 'Conic',
                layoutDependency,
                layoutId: 'e_A3zaxgk',
                style: {
                  background: 'conic-gradient(from 180deg at 50% 50%, #4cf 0deg, #4cf 360deg)',
                  backgroundColor: 'rgb(68, 204, 255)',
                  mask: 'none',
                  WebkitMask: 'none',
                },
                variants: {
                  eY3CsaRlv: {
                    background:
                      'conic-gradient(from 0deg at 50% 50%, rgba(255, 255, 255, 0) 7.208614864864882deg, rgb(255, 255, 255) 342deg)',
                    backgroundColor: 'rgba(0, 0, 0, 0)',
                    mask:
                      'url(\'https://framerusercontent.com/images/pGiXYozQ3mE4cilNOItfe2L2fUA.svg\') alpha no-repeat center / cover add',
                    WebkitMask:
                      'url(\'https://framerusercontent.com/images/pGiXYozQ3mE4cilNOItfe2L2fUA.svg\') alpha no-repeat center / cover add',
                  },
                },
                children: /* @__PURE__ */ _jsx2(motion.div, {
                  className: 'framer-1tle620',
                  'data-framer-name': 'Rounding',
                  layoutDependency,
                  layoutId: 'NXiKKM9np',
                  style: {
                    backgroundColor: 'rgb(255, 255, 255)',
                    borderBottomLeftRadius: 1,
                    borderBottomRightRadius: 1,
                    borderTopLeftRadius: 1,
                    borderTopRightRadius: 1,
                  },
                  transformTemplate: transformTemplate1,
                },),
              },),
            },),
          ],
        },),
      },),
    },),
  },);
},);
var css3 = [
  '@supports (aspect-ratio: 1) { body { --framer-aspect-ratio-supported: auto; } }',
  '.framer-Rde70.framer-1rzbk8y, .framer-Rde70 .framer-1rzbk8y { display: block; }',
  '.framer-Rde70.framer-1v01vfs { align-content: center; align-items: center; cursor: pointer; display: flex; flex-direction: row; flex-wrap: nowrap; gap: 0px; height: 40px; justify-content: center; overflow: visible; padding: 0px; position: relative; width: 240px; }',
  '.framer-Rde70 .framer-bagq5 { -webkit-user-select: none; flex: none; height: auto; position: relative; user-select: none; white-space: pre; width: auto; }',
  '.framer-Rde70 .framer-j7pn8o { aspect-ratio: 1 / 1; flex: none; height: var(--framer-aspect-ratio-supported, 20px); overflow: hidden; position: relative; width: 20px; }',
  '.framer-Rde70 .framer-11aixt1 { bottom: 0px; flex: none; left: 0px; overflow: visible; position: absolute; right: 0px; top: 0px; }',
  '.framer-Rde70 .framer-1tle620 { aspect-ratio: 1 / 1; flex: none; height: var(--framer-aspect-ratio-supported, 2px); left: 50%; overflow: visible; position: absolute; top: 0px; width: 2px; }',
  '@supports (background: -webkit-named-image(i)) and (not (font-palette:dark)) { .framer-Rde70.framer-1v01vfs { gap: 0px; } .framer-Rde70.framer-1v01vfs > * { margin: 0px; margin-left: calc(0px / 2); margin-right: calc(0px / 2); } .framer-Rde70.framer-1v01vfs > :first-child { margin-left: 0px; } .framer-Rde70.framer-1v01vfs > :last-child { margin-right: 0px; } }',
  '.framer-Rde70.framer-v-sqy5j3 .framer-11aixt1 { overflow: hidden; }',
];
var FramerJGT1yLjbN = withCSS(Component, css3, 'framer-Rde70',);
var stdin_default = FramerJGT1yLjbN;
FramerJGT1yLjbN.displayName = 'Submit Button';
FramerJGT1yLjbN.defaultProps = { height: 40, width: 240, };
addPropertyControls2(FramerJGT1yLjbN, {
  variant: {
    options: ['Sc0Dg6vVo', 'eY3CsaRlv', 'Vt9zLruWe', 'dfgg9_FTh', 'MolWXut_w',],
    optionTitles: ['Default', 'Loading', 'Disabled', 'Success', 'Error',],
    title: 'Variant',
    type: ControlType2.Enum,
  },
  Shx5vBWP_: { title: 'Click', type: ControlType2.EventHandler, },
},);
addFonts(FramerJGT1yLjbN, [{
  explicitInter: true,
  fonts: [{
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0460-052F, U+1C80-1C88, U+20B4, U+2DE0-2DFF, U+A640-A69F, U+FE2E-FE2F',
    url: 'https://framerusercontent.com/assets/hyOgCu0Xnghbimh0pE8QTvtt2AU.woff2',
    weight: '600',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0301, U+0400-045F, U+0490-0491, U+04B0-04B1, U+2116',
    url: 'https://framerusercontent.com/assets/NeGmSOXrPBfEFIy5YZeHq17LEDA.woff2',
    weight: '600',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+1F00-1FFF',
    url: 'https://framerusercontent.com/assets/oYaAX5himiTPYuN8vLWnqBbfD2s.woff2',
    weight: '600',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0370-03FF',
    url: 'https://framerusercontent.com/assets/lEJLP4R0yuCaMCjSXYHtJw72M.woff2',
    weight: '600',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0100-024F, U+0259, U+1E00-1EFF, U+2020, U+20A0-20AB, U+20AD-20CF, U+2113, U+2C60-2C7F, U+A720-A7FF',
    url: 'https://framerusercontent.com/assets/cRJyLNuTJR5jbyKzGi33wU9cqIQ.woff2',
    weight: '600',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange:
      'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD',
    url: 'https://framerusercontent.com/assets/1ZFS7N918ojhhd0nQWdj3jz4w.woff2',
    weight: '600',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0102-0103, U+0110-0111, U+0128-0129, U+0168-0169, U+01A0-01A1, U+01AF-01B0, U+1EA0-1EF9, U+20AB',
    url: 'https://framerusercontent.com/assets/A0Wcc7NgXMjUuFdquHDrIZpzZw0.woff2',
    weight: '600',
  },],
},], { supportsExplicitInterCodegen: true, },);

// https :https://framerusercontent.com/modules/LNe7c2MkdJc10yncD9lr/PDFJYR9t4CoWEFQVpcy5/WeRAxkTPv.js
var SubmitButtonFonts = getFonts(stdin_default,);
var serializationHash2 = 'framer-nmpJc';
var variantClassNames2 = { a4MMSry4j: 'framer-v-1f95cno', };
var transition12 = { bounce: 0.2, delay: 0, duration: 0.4, type: 'spring', };
var Transition2 = ({ value, children, },) => {
  const config = React2.useContext(MotionConfigContext2,);
  const transition = value !== null && value !== void 0 ? value : config.transition;
  const contextValue = React2.useMemo(() => ({ ...config, transition, }), [JSON.stringify(transition,),],);
  return /* @__PURE__ */ _jsx3(MotionConfigContext2.Provider, { value: contextValue, children, },);
};
var Variants2 = motion2.create(React2.Fragment,);
var getProps2 = ({ height, id, width, ...props },) => {
  return { ...props, };
};
var createLayoutDependency2 = (props, variants,) => {
  if (props.layoutDependency) return variants.join('-',) + props.layoutDependency;
  return variants.join('-',);
};
var Component2 = /* @__PURE__ */ React2.forwardRef(function (props, ref,) {
  const { activeLocale, setLocale, } = useLocaleInfo2();
  const { style, className: className3, layoutId, variant, MxCOFGtA2, ...restProps } = getProps2(props,);
  const {
    baseVariant,
    classNames,
    clearLoadingGesture,
    gestureHandlers,
    gestureVariant,
    isLoading,
    setGestureState,
    setVariant,
    variants,
  } = useVariantState2({ defaultVariant: 'a4MMSry4j', variant, variantClassNames: variantClassNames2, },);
  const layoutDependency = createLayoutDependency2(props, variants,);
  const { activeVariantCallback, delay, } = useActiveVariantCallback2(baseVariant,);
  const Shx5vBWP_1q5isy = activeVariantCallback(async (...args) => {
    if (MxCOFGtA2) {
      const res = await MxCOFGtA2(...args,);
      if (res === false) return false;
    }
  },);
  const ref1 = React2.useRef(null,);
  const defaultLayoutId = React2.useId();
  const sharedStyleClassNames = [];
  const componentViewport = useComponentViewport2();
  return /* @__PURE__ */ _jsx3(LayoutGroup2, {
    id: layoutId !== null && layoutId !== void 0 ? layoutId : defaultLayoutId,
    children: /* @__PURE__ */ _jsx3(Variants2, {
      animate: variants,
      initial: false,
      children: /* @__PURE__ */ _jsx3(Transition2, {
        value: transition12,
        children: /* @__PURE__ */ _jsxs2(motion2.div, {
          ...restProps,
          ...gestureHandlers,
          className: cx2(serializationHash2, ...sharedStyleClassNames, 'framer-1f95cno', className3, classNames,),
          'data-framer-name': 'Variant 1',
          layoutDependency,
          layoutId: 'a4MMSry4j',
          ref: ref !== null && ref !== void 0 ? ref : ref1,
          style: { ...style, },
          children: [
            /* @__PURE__ */ _jsxs2(motion2.label, {
              className: 'framer-18b8b8o',
              layoutDependency,
              layoutId: 'Bt9K87jjT',
              children: [
                /* @__PURE__ */ _jsx3(RichText2, {
                  __fromCanvasComponent: true,
                  children: /* @__PURE__ */ _jsx3(React2.Fragment, {
                    children: /* @__PURE__ */ _jsx3(motion2.p, {
                      style: {
                        '--font-selector': 'SW50ZXItTWVkaXVt',
                        '--framer-font-family': '"Inter", "Inter Placeholder", sans-serif',
                        '--framer-font-size': '12px',
                        '--framer-font-weight': '500',
                        '--framer-text-color': 'var(--extracted-r6o4lv, rgb(136, 136, 136))',
                      },
                      children: 'Email',
                    },),
                  },),
                  className: 'framer-499mo0',
                  fonts: ['Inter-Medium',],
                  layoutDependency,
                  layoutId: 'SHdD67Bgb',
                  style: { '--extracted-r6o4lv': 'rgb(136, 136, 136)', },
                  verticalAlignment: 'top',
                  withExternalLayout: true,
                },),
                /* @__PURE__ */ _jsx3(FormPlainTextInput, {
                  className: 'framer-hpyyb2',
                  inputName: 'Email',
                  layoutDependency,
                  layoutId: 'Q4lXvxc93',
                  placeholder: 'Email Address',
                  style: {
                    '--framer-input-background': 'rgba(187, 187, 187, 0.15)',
                    '--framer-input-border-bottom-width': '1px',
                    '--framer-input-border-color': 'rgba(136, 136, 136, 0.1)',
                    '--framer-input-border-left-width': '1px',
                    '--framer-input-border-radius-bottom-left': '10px',
                    '--framer-input-border-radius-bottom-right': '10px',
                    '--framer-input-border-radius-top-left': '10px',
                    '--framer-input-border-radius-top-right': '10px',
                    '--framer-input-border-right-width': '1px',
                    '--framer-input-border-style': 'solid',
                    '--framer-input-border-top-width': '1px',
                    '--framer-input-font-color': 'rgb(153, 153, 153)',
                    '--framer-input-icon-color': 'rgb(153, 153, 153)',
                    '--framer-input-placeholder-color': 'rgb(153, 153, 153)',
                  },
                  type: 'email',
                },),
              ],
            },),
            /* @__PURE__ */ _jsxs2(motion2.label, {
              className: 'framer-18z0sf3',
              layoutDependency,
              layoutId: 'k3cxHIEme',
              children: [
                /* @__PURE__ */ _jsx3(RichText2, {
                  __fromCanvasComponent: true,
                  children: /* @__PURE__ */ _jsx3(React2.Fragment, {
                    children: /* @__PURE__ */ _jsx3(motion2.p, {
                      style: {
                        '--font-selector': 'SW50ZXItTWVkaXVt',
                        '--framer-font-family': '"Inter", "Inter Placeholder", sans-serif',
                        '--framer-font-size': '12px',
                        '--framer-font-weight': '500',
                        '--framer-text-color': 'var(--extracted-r6o4lv, rgb(136, 136, 136))',
                      },
                      children: 'How can we help?',
                    },),
                  },),
                  className: 'framer-135m04j',
                  fonts: ['Inter-Medium',],
                  layoutDependency,
                  layoutId: 'yuhYlqafz',
                  style: { '--extracted-r6o4lv': 'rgb(136, 136, 136)', },
                  verticalAlignment: 'top',
                  withExternalLayout: true,
                },),
                /* @__PURE__ */ _jsx3(FormPlainTextInput, {
                  className: 'framer-ziid8k',
                  inputName: 'Text',
                  layoutDependency,
                  layoutId: 'iD0sImSHJ',
                  placeholder: 'I would like Rivet to help solve for my company...',
                  required: true,
                  style: {
                    '--framer-input-background': 'rgba(187, 187, 187, 0.15)',
                    '--framer-input-border-bottom-width': '1px',
                    '--framer-input-border-color': 'rgba(136, 136, 136, 0.1)',
                    '--framer-input-border-left-width': '1px',
                    '--framer-input-border-radius-bottom-left': '10px',
                    '--framer-input-border-radius-bottom-right': '10px',
                    '--framer-input-border-radius-top-left': '10px',
                    '--framer-input-border-radius-top-right': '10px',
                    '--framer-input-border-right-width': '1px',
                    '--framer-input-border-style': 'solid',
                    '--framer-input-border-top-width': '1px',
                    '--framer-input-font-color': 'rgb(153, 153, 153)',
                    '--framer-input-icon-color': 'rgb(153, 153, 153)',
                    '--framer-input-placeholder-color': 'rgb(153, 153, 153)',
                  },
                  type: 'textarea',
                },),
              ],
            },),
            /* @__PURE__ */ _jsx3(ComponentViewportProvider, {
              height: 40,
              width: (componentViewport === null || componentViewport === void 0 ? void 0 : componentViewport.width) || '100vw',
              children: /* @__PURE__ */ _jsx3(motion2.div, {
                className: 'framer-1ff05f1-container',
                layoutDependency,
                layoutId: 'QJHd6hxCj-container',
                children: /* @__PURE__ */ _jsx3(stdin_default, {
                  height: '100%',
                  id: 'QJHd6hxCj',
                  layoutId: 'QJHd6hxCj',
                  Shx5vBWP_: Shx5vBWP_1q5isy,
                  style: { height: '100%', width: '100%', },
                  variant: 'Sc0Dg6vVo',
                  width: '100%',
                },),
              },),
            },),
            /* @__PURE__ */ _jsx3(RichText2, {
              __fromCanvasComponent: true,
              children: /* @__PURE__ */ _jsx3(React2.Fragment, {
                children: /* @__PURE__ */ _jsxs2(motion2.h2, {
                  style: {
                    '--font-selector': 'SW50ZXItTWVkaXVt',
                    '--framer-font-family': '"Inter", "Inter Placeholder", sans-serif',
                    '--framer-font-weight': '500',
                    '--framer-letter-spacing': '-1px',
                    '--framer-text-alignment': 'left',
                    '--framer-text-color': 'var(--extracted-1of0zx5, rgb(138, 138, 138))',
                  },
                  children: [
                    'You can also email us at ',
                    /* @__PURE__ */ _jsx3(motion2.span, {
                      style: { '--font-selector': 'SW50ZXItQm9sZA==', '--framer-font-weight': '700', },
                      children: 'sales@rivet.gg',
                    },),
                  ],
                },),
              },),
              className: 'framer-b5wt5e',
              fonts: ['Inter-Medium', 'Inter-Bold',],
              layoutDependency,
              layoutId: 'F6HgtMCda',
              style: { '--extracted-1of0zx5': 'rgb(138, 138, 138)', '--framer-paragraph-spacing': '0px', },
              verticalAlignment: 'top',
              withExternalLayout: true,
            },),
          ],
        },),
      },),
    },),
  },);
},);
var css4 = [
  '@supports (aspect-ratio: 1) { body { --framer-aspect-ratio-supported: auto; } }',
  '.framer-nmpJc.framer-1leiklf, .framer-nmpJc .framer-1leiklf { display: block; }',
  '.framer-nmpJc.framer-1f95cno { align-content: center; align-items: center; display: flex; flex-direction: column; flex-wrap: nowrap; gap: 10px; height: min-content; justify-content: center; padding: 0px; position: relative; width: min-content; }',
  '.framer-nmpJc .framer-18b8b8o, .framer-nmpJc .framer-18z0sf3 { align-content: flex-start; align-items: flex-start; align-self: stretch; display: flex; flex: none; flex-direction: column; flex-wrap: nowrap; gap: 10px; height: min-content; justify-content: flex-start; padding: 0px; position: relative; width: auto; }',
  '.framer-nmpJc .framer-499mo0, .framer-nmpJc .framer-135m04j { flex: none; height: auto; position: relative; white-space: pre; width: auto; }',
  '.framer-nmpJc .framer-hpyyb2 { --framer-input-focused-border-color: #0099ff; --framer-input-focused-border-style: solid; --framer-input-focused-border-width: 1px; --framer-input-font-family: "Inter"; --framer-input-font-letter-spacing: 0em; --framer-input-font-line-height: 1.2em; --framer-input-font-size: 14px; --framer-input-font-weight: 400; --framer-input-padding: 12px; flex: none; height: 40px; position: relative; width: 100%; }',
  '.framer-nmpJc .framer-ziid8k { --framer-input-focused-border-color: #0099ff; --framer-input-focused-border-style: solid; --framer-input-focused-border-width: 1px; --framer-input-font-family: "Inter"; --framer-input-font-letter-spacing: 0em; --framer-input-font-line-height: 1.2em; --framer-input-font-size: 14px; --framer-input-font-weight: 400; --framer-input-padding: 12px; --framer-input-wrapper-height: auto; --framer-textarea-resize: vertical; flex: none; height: auto; min-height: 173px; position: relative; width: 100%; }',
  '.framer-nmpJc .framer-1ff05f1-container { align-self: stretch; flex: none; height: 40px; position: relative; width: auto; }',
  '.framer-nmpJc .framer-b5wt5e { flex: none; height: 25px; overflow: visible; position: relative; white-space: pre-wrap; width: 320px; word-break: break-word; word-wrap: break-word; }',
  '@supports (background: -webkit-named-image(i)) and (not (font-palette:dark)) { .framer-nmpJc.framer-1f95cno, .framer-nmpJc .framer-18b8b8o, .framer-nmpJc .framer-18z0sf3 { gap: 0px; } .framer-nmpJc.framer-1f95cno > *, .framer-nmpJc .framer-18b8b8o > *, .framer-nmpJc .framer-18z0sf3 > * { margin: 0px; margin-bottom: calc(10px / 2); margin-top: calc(10px / 2); } .framer-nmpJc.framer-1f95cno > :first-child, .framer-nmpJc .framer-18b8b8o > :first-child, .framer-nmpJc .framer-18z0sf3 > :first-child { margin-top: 0px; } .framer-nmpJc.framer-1f95cno > :last-child, .framer-nmpJc .framer-18b8b8o > :last-child, .framer-nmpJc .framer-18z0sf3 > :last-child { margin-bottom: 0px; } }',
];
var FramerWeRAxkTPv = withCSS2(Component2, css4, 'framer-nmpJc',);
var stdin_default2 = FramerWeRAxkTPv;
FramerWeRAxkTPv.displayName = 'Contact From Fields';
FramerWeRAxkTPv.defaultProps = { height: 357, width: 320, };
addFonts2(FramerWeRAxkTPv, [{
  explicitInter: true,
  fonts: [{
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0460-052F, U+1C80-1C88, U+20B4, U+2DE0-2DFF, U+A640-A69F, U+FE2E-FE2F',
    url: 'https://framerusercontent.com/assets/5A3Ce6C9YYmCjpQx9M4inSaKU.woff2',
    weight: '500',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0301, U+0400-045F, U+0490-0491, U+04B0-04B1, U+2116',
    url: 'https://framerusercontent.com/assets/Qx95Xyt0Ka3SGhinnbXIGpEIyP4.woff2',
    weight: '500',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+1F00-1FFF',
    url: 'https://framerusercontent.com/assets/6mJuEAguuIuMog10gGvH5d3cl8.woff2',
    weight: '500',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0370-03FF',
    url: 'https://framerusercontent.com/assets/xYYWaj7wCU5zSQH0eXvSaS19wo.woff2',
    weight: '500',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0100-024F, U+0259, U+1E00-1EFF, U+2020, U+20A0-20AB, U+20AD-20CF, U+2113, U+2C60-2C7F, U+A720-A7FF',
    url: 'https://framerusercontent.com/assets/otTaNuNpVK4RbdlT7zDDdKvQBA.woff2',
    weight: '500',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange:
      'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD',
    url: 'https://framerusercontent.com/assets/d3tHnaQIAeqiE5hGcRw4mmgWYU.woff2',
    weight: '500',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0102-0103, U+0110-0111, U+0128-0129, U+0168-0169, U+01A0-01A1, U+01AF-01B0, U+1EA0-1EF9, U+20AB',
    url: 'https://framerusercontent.com/assets/DolVirEGb34pEXEp8t8FQBSK4.woff2',
    weight: '500',
  }, {
    family: 'Inter',
    source: 'google',
    style: 'normal',
    url: 'https://fonts.gstatic.com/s/inter/v18/UcCO3FwrK3iLTeHuS_nVMrMxCp50SjIw2boKoduKmMEVuLyfMZ1rib2Bg-4.woff2',
    weight: '400',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0460-052F, U+1C80-1C88, U+20B4, U+2DE0-2DFF, U+A640-A69F, U+FE2E-FE2F',
    url: 'https://framerusercontent.com/assets/DpPBYI0sL4fYLgAkX8KXOPVt7c.woff2',
    weight: '700',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0301, U+0400-045F, U+0490-0491, U+04B0-04B1, U+2116',
    url: 'https://framerusercontent.com/assets/4RAEQdEOrcnDkhHiiCbJOw92Lk.woff2',
    weight: '700',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+1F00-1FFF',
    url: 'https://framerusercontent.com/assets/1K3W8DizY3v4emK8Mb08YHxTbs.woff2',
    weight: '700',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0370-03FF',
    url: 'https://framerusercontent.com/assets/tUSCtfYVM1I1IchuyCwz9gDdQ.woff2',
    weight: '700',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0100-024F, U+0259, U+1E00-1EFF, U+2020, U+20A0-20AB, U+20AD-20CF, U+2113, U+2C60-2C7F, U+A720-A7FF',
    url: 'https://framerusercontent.com/assets/VgYFWiwsAC5OYxAycRXXvhze58.woff2',
    weight: '700',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange:
      'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD',
    url: 'https://framerusercontent.com/assets/DXD0Q7LSl7HEvDzucnyLnGBHM.woff2',
    weight: '700',
  }, {
    family: 'Inter',
    source: 'framer',
    style: 'normal',
    unicodeRange: 'U+0102-0103, U+0110-0111, U+0128-0129, U+0168-0169, U+01A0-01A1, U+01AF-01B0, U+1EA0-1EF9, U+20AB',
    url: 'https://framerusercontent.com/assets/GIryZETIX4IFypco5pYZONKhJIo.woff2',
    weight: '700',
  },],
}, ...SubmitButtonFonts,], { supportsExplicitInterCodegen: true, },);

// https :https://framerusercontent.com/modules/ow6h1b6wQLBemft55pFs/fiMzU6TRqHLGNmiUHMjg/XndMdburz.js
var ContactFromFieldsFonts = getFonts2(stdin_default2,);
var PostHogFormFonts = getFonts2(PostHogForm,);
var cycleOrder2 = ['O8WVLLIhr', 'b1yZ1L01r', 'cNvPvbAJ7',];
var serializationHash3 = 'framer-WMcas';
var variantClassNames3 = { b1yZ1L01r: 'framer-v-1iun8oo', cNvPvbAJ7: 'framer-v-8ozx54', O8WVLLIhr: 'framer-v-1jjbo3t', };
function addPropertyOverrides2(overrides, ...variants) {
  const nextOverrides = {};
  variants === null || variants === void 0
    ? void 0
    : variants.forEach((variant,) => variant && Object.assign(nextOverrides, overrides[variant],));
  return nextOverrides;
}
var transition13 = { bounce: 0.2, delay: 0, duration: 0.4, type: 'spring', };
var Transition3 = ({ value, children, },) => {
  const config = React3.useContext(MotionConfigContext3,);
  const transition = value !== null && value !== void 0 ? value : config.transition;
  const contextValue = React3.useMemo(() => ({ ...config, transition, }), [JSON.stringify(transition,),],);
  return /* @__PURE__ */ _jsx4(MotionConfigContext3.Provider, { value: contextValue, children, },);
};
var Variants3 = motion3.create(React3.Fragment,);
var humanReadableVariantMap2 = { Desktop: 'O8WVLLIhr', Phone: 'b1yZ1L01r', Tablet: 'cNvPvbAJ7', };
var getProps3 = ({ height, id, width, ...props },) => {
  var _humanReadableVariantMap_props_variant, _ref;
  return {
    ...props,
    variant:
      (_ref =
            (_humanReadableVariantMap_props_variant = humanReadableVariantMap2[props.variant]) !== null &&
              _humanReadableVariantMap_props_variant !== void 0
              ? _humanReadableVariantMap_props_variant
              : props.variant) !== null && _ref !== void 0
        ? _ref
        : 'O8WVLLIhr',
  };
};
var createLayoutDependency3 = (props, variants,) => {
  if (props.layoutDependency) return variants.join('-',) + props.layoutDependency;
  return variants.join('-',);
};
var Component3 = /* @__PURE__ */ React3.forwardRef(function (props, ref,) {
  const { activeLocale, setLocale, } = useLocaleInfo3();
  const { style, className: className3, layoutId, variant, ...restProps } = getProps3(props,);
  const {
    baseVariant,
    classNames,
    clearLoadingGesture,
    gestureHandlers,
    gestureVariant,
    isLoading,
    setGestureState,
    setVariant,
    variants,
  } = useVariantState3({ cycleOrder: cycleOrder2, defaultVariant: 'O8WVLLIhr', variant, variantClassNames: variantClassNames3, },);
  const layoutDependency = createLayoutDependency3(props, variants,);
  const ref1 = React3.useRef(null,);
  const defaultLayoutId = React3.useId();
  const sharedStyleClassNames = [className2, className,];
  const componentViewport = useComponentViewport3();
  return /* @__PURE__ */ _jsx4(LayoutGroup3, {
    id: layoutId !== null && layoutId !== void 0 ? layoutId : defaultLayoutId,
    children: /* @__PURE__ */ _jsx4(Variants3, {
      animate: variants,
      initial: false,
      children: /* @__PURE__ */ _jsx4(Transition3, {
        value: transition13,
        children: /* @__PURE__ */ _jsxs3(motion3.div, {
          ...restProps,
          ...gestureHandlers,
          className: cx3(serializationHash3, ...sharedStyleClassNames, 'framer-1jjbo3t', className3, classNames,),
          'data-framer-name': 'Desktop',
          layoutDependency,
          layoutId: 'O8WVLLIhr',
          ref: ref !== null && ref !== void 0 ? ref : ref1,
          style: { ...style, },
          ...addPropertyOverrides2(
            { b1yZ1L01r: { 'data-framer-name': 'Phone', }, cNvPvbAJ7: { 'data-framer-name': 'Tablet', }, },
            baseVariant,
            gestureVariant,
          ),
          children: [
            /* @__PURE__ */ _jsxs3(motion3.div, {
              className: 'framer-1v1ta5w',
              layoutDependency,
              layoutId: 'wgy60thcI',
              children: [
                /* @__PURE__ */ _jsxs3(motion3.div, {
                  className: 'framer-oe8dn1',
                  layoutDependency,
                  layoutId: 'Hno0cIgYo',
                  children: [
                    /* @__PURE__ */ _jsx4(RichText3, {
                      __fromCanvasComponent: true,
                      children: /* @__PURE__ */ _jsx4(React3.Fragment, {
                        children: /* @__PURE__ */ _jsx4(motion3.h3, {
                          className: 'framer-styles-preset-jttjmp',
                          'data-styles-preset': 'zu841OiIg',
                          children: 'Contact our Sales Team.',
                        },),
                      },),
                      className: 'framer-1m96p32',
                      fonts: ['Inter',],
                      layoutDependency,
                      layoutId: 'ZSgKNhLIw',
                      style: {
                        '--framer-link-text-color': 'rgb(0, 153, 255)',
                        '--framer-link-text-decoration': 'underline',
                        '--framer-paragraph-spacing': '0px',
                      },
                      verticalAlignment: 'top',
                      withExternalLayout: true,
                    },),
                    /* @__PURE__ */ _jsx4(RichText3, {
                      __fromCanvasComponent: true,
                      children: /* @__PURE__ */ _jsx4(React3.Fragment, {
                        children: /* @__PURE__ */ _jsx4(motion3.h2, {
                          className: 'framer-styles-preset-az499w',
                          'data-styles-preset': 'kHb0JRZSH',
                          children: 'Get a demo, tailored pricing to fit your needs, or have your questions answered about Rivet.',
                        },),
                      },),
                      className: 'framer-10s5a9w',
                      fonts: ['Inter',],
                      layoutDependency,
                      layoutId: 'bg3XzG9rS',
                      style: { '--framer-paragraph-spacing': '0px', },
                      verticalAlignment: 'top',
                      withExternalLayout: true,
                    },),
                  ],
                },),
                /* @__PURE__ */ _jsxs3(motion3.div, {
                  className: 'framer-15zinc2',
                  layoutDependency,
                  layoutId: 'KNfHfMqBr',
                  children: [
                    /* @__PURE__ */ _jsx4(RichText3, {
                      __fromCanvasComponent: true,
                      children: /* @__PURE__ */ _jsx4(React3.Fragment, {
                        children: /* @__PURE__ */ _jsx4(motion3.h2, {
                          className: 'framer-styles-preset-az499w',
                          'data-styles-preset': 'kHb0JRZSH',
                          style: { '--framer-text-color': 'var(--extracted-1of0zx5, rgb(255, 255, 255))', },
                          children: 'Email Support',
                        },),
                      },),
                      className: 'framer-rgl66s',
                      fonts: ['Inter',],
                      layoutDependency,
                      layoutId: 'PCNVeIHLM',
                      style: { '--extracted-1of0zx5': 'rgb(255, 255, 255)', '--framer-paragraph-spacing': '0px', },
                      verticalAlignment: 'top',
                      withExternalLayout: true,
                    },),
                    /* @__PURE__ */ _jsx4(RichText3, {
                      __fromCanvasComponent: true,
                      children: /* @__PURE__ */ _jsx4(React3.Fragment, {
                        children: /* @__PURE__ */ _jsx4(motion3.h2, {
                          className: 'framer-styles-preset-az499w',
                          'data-styles-preset': 'kHb0JRZSH',
                          children: 'support@rivet.gg',
                        },),
                      },),
                      className: 'framer-js62il',
                      fonts: ['Inter',],
                      layoutDependency,
                      layoutId: 'gD6EhkKso',
                      style: { '--framer-paragraph-spacing': '0px', },
                      verticalAlignment: 'top',
                      withExternalLayout: true,
                    },),
                  ],
                },),
              ],
            },),
            /* @__PURE__ */ _jsx4(ComponentViewportProvider2, {
              children: /* @__PURE__ */ _jsx4(motion3.div, {
                className: 'framer-1qerw4k-container',
                layoutDependency,
                layoutId: 'wHRmgvLzX-container',
                children: /* @__PURE__ */ _jsx4(PostHogForm, {
                  height: '100%',
                  id: 'wHRmgvLzX',
                  layoutId: 'wHRmgvLzX',
                  width: '100%',
                  children: /* @__PURE__ */ _jsx4(ComponentViewportProvider2, {
                    height: 357,
                    width: '320px',
                    children: /* @__PURE__ */ _jsx4(motion3.div, {
                      className: 'framer-1lrzgz8-container',
                      layoutDependency,
                      layoutId: 'r8oX0thUc-container',
                      children: /* @__PURE__ */ _jsx4(stdin_default2, {
                        height: '100%',
                        id: 'r8oX0thUc',
                        layoutId: 'r8oX0thUc',
                        width: '100%',
                      },),
                    },),
                  },),
                },),
              },),
            },),
          ],
        },),
      },),
    },),
  },);
},);
var css5 = [
  '@supports (aspect-ratio: 1) { body { --framer-aspect-ratio-supported: auto; } }',
  '.framer-WMcas.framer-c8kegl, .framer-WMcas .framer-c8kegl { display: block; }',
  '.framer-WMcas.framer-1jjbo3t { align-content: flex-start; align-items: flex-start; display: flex; flex-direction: row; flex-wrap: nowrap; gap: 166px; height: min-content; justify-content: center; overflow: hidden; padding: 32px; position: relative; width: 1200px; }',
  '.framer-WMcas .framer-1v1ta5w { align-content: flex-start; align-items: flex-start; display: flex; flex: none; flex-direction: column; flex-wrap: nowrap; gap: 52px; height: min-content; justify-content: center; overflow: hidden; padding: 0px 0px 46px 0px; position: relative; width: 30%; }',
  '.framer-WMcas .framer-oe8dn1, .framer-WMcas .framer-15zinc2 { align-content: flex-start; align-items: flex-start; display: flex; flex: none; flex-direction: column; flex-wrap: nowrap; gap: 10px; height: min-content; justify-content: center; overflow: hidden; padding: 0px; position: relative; width: 100%; }',
  '.framer-WMcas .framer-1m96p32 { flex: none; height: auto; overflow: visible; position: relative; white-space: pre-wrap; width: 354px; word-break: break-word; word-wrap: break-word; }',
  '.framer-WMcas .framer-10s5a9w { flex: none; height: 79px; overflow: visible; position: relative; white-space: pre-wrap; width: 320px; word-break: break-word; word-wrap: break-word; }',
  '.framer-WMcas .framer-rgl66s, .framer-WMcas .framer-js62il { flex: none; height: 25px; overflow: visible; position: relative; white-space: pre-wrap; width: 320px; word-break: break-word; word-wrap: break-word; }',
  '.framer-WMcas .framer-1qerw4k-container { flex: none; height: auto; min-width: 100px; position: relative; width: auto; }',
  '.framer-WMcas .framer-1lrzgz8-container { height: auto; position: relative; width: auto; }',
  '@supports (background: -webkit-named-image(i)) and (not (font-palette:dark)) { .framer-WMcas.framer-1jjbo3t, .framer-WMcas .framer-1v1ta5w, .framer-WMcas .framer-oe8dn1, .framer-WMcas .framer-15zinc2 { gap: 0px; } .framer-WMcas.framer-1jjbo3t > * { margin: 0px; margin-left: calc(166px / 2); margin-right: calc(166px / 2); } .framer-WMcas.framer-1jjbo3t > :first-child { margin-left: 0px; } .framer-WMcas.framer-1jjbo3t > :last-child { margin-right: 0px; } .framer-WMcas .framer-1v1ta5w > * { margin: 0px; margin-bottom: calc(52px / 2); margin-top: calc(52px / 2); } .framer-WMcas .framer-1v1ta5w > :first-child, .framer-WMcas .framer-oe8dn1 > :first-child, .framer-WMcas .framer-15zinc2 > :first-child { margin-top: 0px; } .framer-WMcas .framer-1v1ta5w > :last-child, .framer-WMcas .framer-oe8dn1 > :last-child, .framer-WMcas .framer-15zinc2 > :last-child { margin-bottom: 0px; } .framer-WMcas .framer-oe8dn1 > *, .framer-WMcas .framer-15zinc2 > * { margin: 0px; margin-bottom: calc(10px / 2); margin-top: calc(10px / 2); } }',
  '.framer-WMcas.framer-v-1iun8oo.framer-1jjbo3t { align-content: center; align-items: center; flex-direction: column; gap: 16px; width: 522px; }',
  '.framer-WMcas.framer-v-1iun8oo .framer-1v1ta5w { align-content: center; align-items: center; gap: 16px; width: 100%; }',
  '@supports (background: -webkit-named-image(i)) and (not (font-palette:dark)) { .framer-WMcas.framer-v-1iun8oo.framer-1jjbo3t, .framer-WMcas.framer-v-1iun8oo .framer-1v1ta5w { gap: 0px; } .framer-WMcas.framer-v-1iun8oo.framer-1jjbo3t > *, .framer-WMcas.framer-v-1iun8oo .framer-1v1ta5w > * { margin: 0px; margin-bottom: calc(16px / 2); margin-top: calc(16px / 2); } .framer-WMcas.framer-v-1iun8oo.framer-1jjbo3t > :first-child, .framer-WMcas.framer-v-1iun8oo .framer-1v1ta5w > :first-child { margin-top: 0px; } .framer-WMcas.framer-v-1iun8oo.framer-1jjbo3t > :last-child, .framer-WMcas.framer-v-1iun8oo .framer-1v1ta5w > :last-child { margin-bottom: 0px; } }',
  ...css2,
  ...css,
];
var FramerXndMdburz = withCSS3(Component3, css5, 'framer-WMcas',);
var stdin_default3 = FramerXndMdburz;
FramerXndMdburz.displayName = 'Sales (Page)';
FramerXndMdburz.defaultProps = { height: 421, width: 1200, };
addPropertyControls3(FramerXndMdburz, {
  variant: {
    options: ['O8WVLLIhr', 'b1yZ1L01r', 'cNvPvbAJ7',],
    optionTitles: ['Desktop', 'Phone', 'Tablet',],
    title: 'Variant',
    type: ControlType3.Enum,
  },
},);
addFonts3(FramerXndMdburz, [
  {
    explicitInter: true,
    fonts: [{
      family: 'Inter',
      source: 'framer',
      style: 'normal',
      unicodeRange: 'U+0460-052F, U+1C80-1C88, U+20B4, U+2DE0-2DFF, U+A640-A69F, U+FE2E-FE2F',
      url: 'https://framerusercontent.com/assets/5vvr9Vy74if2I6bQbJvbw7SY1pQ.woff2',
      weight: '400',
    }, {
      family: 'Inter',
      source: 'framer',
      style: 'normal',
      unicodeRange: 'U+0301, U+0400-045F, U+0490-0491, U+04B0-04B1, U+2116',
      url: 'https://framerusercontent.com/assets/EOr0mi4hNtlgWNn9if640EZzXCo.woff2',
      weight: '400',
    }, {
      family: 'Inter',
      source: 'framer',
      style: 'normal',
      unicodeRange: 'U+1F00-1FFF',
      url: 'https://framerusercontent.com/assets/Y9k9QrlZAqio88Klkmbd8VoMQc.woff2',
      weight: '400',
    }, {
      family: 'Inter',
      source: 'framer',
      style: 'normal',
      unicodeRange: 'U+0370-03FF',
      url: 'https://framerusercontent.com/assets/OYrD2tBIBPvoJXiIHnLoOXnY9M.woff2',
      weight: '400',
    }, {
      family: 'Inter',
      source: 'framer',
      style: 'normal',
      unicodeRange: 'U+0100-024F, U+0259, U+1E00-1EFF, U+2020, U+20A0-20AB, U+20AD-20CF, U+2113, U+2C60-2C7F, U+A720-A7FF',
      url: 'https://framerusercontent.com/assets/JeYwfuaPfZHQhEG8U5gtPDZ7WQ.woff2',
      weight: '400',
    }, {
      family: 'Inter',
      source: 'framer',
      style: 'normal',
      unicodeRange:
        'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD',
      url: 'https://framerusercontent.com/assets/vQyevYAyHtARFwPqUzQGpnDs.woff2',
      weight: '400',
    }, {
      family: 'Inter',
      source: 'framer',
      style: 'normal',
      unicodeRange: 'U+0102-0103, U+0110-0111, U+0128-0129, U+0168-0169, U+01A0-01A1, U+01AF-01B0, U+1EA0-1EF9, U+20AB',
      url: 'https://framerusercontent.com/assets/b6Y37FthZeALduNqHicBT6FutY.woff2',
      weight: '400',
    },],
  },
  ...ContactFromFieldsFonts,
  ...PostHogFormFonts,
  ...getFontsFromSharedStyle(fonts2,),
  ...getFontsFromSharedStyle(fonts,),
], { supportsExplicitInterCodegen: true, },);

// virtual:sales
import { WithFramerBreakpoints, } from 'unframer';
import { jsx, } from 'react/jsx-runtime';
stdin_default3.Responsive = (props,) => {
  return /* @__PURE__ */ jsx(WithFramerBreakpoints, { Component: stdin_default3, ...props, },);
};
var sales_default = stdin_default3;
export { sales_default as default, };
