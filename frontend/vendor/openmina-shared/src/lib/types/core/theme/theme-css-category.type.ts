export interface ThemeCssCategory {
  base: { [p: BaseCssVariable]: string };
  success: { [p: SuccessCssVariable]: string };
  aware: { [p: AwareCssVariable]: string };
  warn: { [p: WarnCssVariable]: string };
  selected: { [p: SelectedCssVariable]: string };
  special: { [p: SpecialCssVariable]: string };
  chart: { [p: ChartCssVariable]: string };
  code: { [p: CodeCssVariable]: string };
  cta: { [p: CtaCssVariable]: string };
}

export const BASE_CSS_PREFIX = '--base-';
export const SUCCESS_CSS_PREFIX = '--success-';
export const AWARE_CSS_PREFIX = '--aware-';
export const WARN_CSS_PREFIX = '--warn-';
export const SELECTED_CSS_PREFIX = '--selected-';
export const SPECIAL_CSS_PREFIX = '--special-';
export const CHART_CSS_PREFIX = '--chart-';
export const CODE_CSS_PREFIX = '--code-';
export const CTA_CSS_PREFIX = '--cta-';

type BaseCssVariable = `${typeof BASE_CSS_PREFIX}${string}`;
type SuccessCssVariable = `${typeof SUCCESS_CSS_PREFIX}${string}`;
type AwareCssVariable = `${typeof AWARE_CSS_PREFIX}${string}`;
type WarnCssVariable = `${typeof WARN_CSS_PREFIX}${string}`;
type SelectedCssVariable = `${typeof SELECTED_CSS_PREFIX}${string}`;
type SpecialCssVariable = `${typeof SPECIAL_CSS_PREFIX}${string}`;
type ChartCssVariable = `${typeof CHART_CSS_PREFIX}${string}`;
type CodeCssVariable = `${typeof CODE_CSS_PREFIX}${string}`;
type CtaCssVariable = `${typeof CTA_CSS_PREFIX}${string}`;
