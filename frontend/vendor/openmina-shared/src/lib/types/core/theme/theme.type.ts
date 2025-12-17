import { ThemeType } from './theme-types.type';
import { ThemeCssCategory } from './theme-css-category.type';

export interface Theme {
  name: ThemeType;
  categories: ThemeCssCategory;
}
