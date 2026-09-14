// SPDX-License-Identifier: Apache-2.0

pub use dene_macros::Refine;

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(Default)]
pub enum TextAlign {
  #[default]
  Left,
  Center,
  Right,
}

pub trait IsEmpty {
  fn is_empty(&self) -> bool;
}

pub trait Refine: Sized {
  type Refinement;

  fn refine(&mut self, refinement: &Self::Refinement);

  fn refined(mut self, refinement: Self::Refinement) -> Self {
    self.refine(&refinement);
    self
  }
}

macro_rules! geometry_refinement {
  ($refinement:ident, $geometry:ident, $($field:ident),+ $(,)?) => {
    #[derive(Debug, Clone)]
    pub struct $refinement<T> {
      $(pub $field: Option<T>,)+
    }

    impl<T> Default for $refinement<T> {
      fn default() -> Self {
        Self {
          $($field: None,)+
        }
      }
    }

    impl<T> IsEmpty for $refinement<T> {
      fn is_empty(&self) -> bool {
        true $(&& self.$field.is_none())+
      }
    }

    impl<T: Clone> Refine for taffy::$geometry<T> {
      type Refinement = $refinement<T>;

      fn refine(&mut self, refinement: &Self::Refinement) {
        $(
          if let Some(value) = &refinement.$field {
            self.$field = value.clone();
          }
        )+
      }
    }

    impl<T: Clone> Refine for $refinement<T> {
      type Refinement = Self;

      fn refine(&mut self, refinement: &Self::Refinement) {
        $(
          if refinement.$field.is_some() {
            self.$field = refinement.$field.clone();
          }
        )+
      }
    }

    impl<T> From<taffy::$geometry<T>> for $refinement<T> {
      fn from(geometry: taffy::$geometry<T>) -> Self {
        Self {
          $($field: Some(geometry.$field),)+
        }
      }
    }
  };
}

geometry_refinement!(PointRefinement, Point, x, y);
geometry_refinement!(SizeRefinement, Size, width, height);
geometry_refinement!(RectRefinement, Rect, left, right, top, bottom);
geometry_refinement!(LineRefinement, Line, start, end);

#[derive(Debug, Clone, Refine)]
pub struct Style {
  pub display: taffy::Display,
  pub item_is_table: bool,
  pub item_is_replaced: bool,
  pub box_sizing: taffy::BoxSizing,
  pub direction: taffy::Direction,

  #[refine]
  pub overflow: taffy::Point<taffy::Overflow>,
  pub scrollbar_width: f32,

  pub position: taffy::Position,
  #[refine]
  pub inset: taffy::Rect<taffy::LengthPercentageAuto>,

  #[refine]
  pub size: taffy::Size<taffy::Dimension>,
  #[refine]
  pub min_size: taffy::Size<taffy::Dimension>,
  #[refine]
  pub max_size: taffy::Size<taffy::Dimension>,
  pub aspect_ratio: Option<f32>,

  #[refine]
  pub margin: taffy::Rect<taffy::LengthPercentageAuto>,
  #[refine]
  pub padding: taffy::Rect<taffy::LengthPercentage>,
  #[refine]
  pub border: taffy::Rect<taffy::LengthPercentage>,

  pub align_items: Option<taffy::AlignItems>,
  pub align_self: Option<taffy::AlignSelf>,
  pub justify_items: Option<taffy::JustifyItems>,
  pub justify_self: Option<taffy::JustifySelf>,
  pub align_content: Option<taffy::AlignContent>,
  pub justify_content: Option<taffy::JustifyContent>,
  #[refine]
  pub gap: taffy::Size<taffy::LengthPercentage>,

  pub text_align: taffy::TextAlign,

  pub flex_direction: taffy::FlexDirection,
  pub flex_wrap: taffy::FlexWrap,
  pub flex_basis: taffy::Dimension,
  pub flex_grow: f32,
  pub flex_shrink: f32,

  pub grid_template_rows: Vec<taffy::GridTemplateComponent<String>>,
  pub grid_template_columns: Vec<taffy::GridTemplateComponent<String>>,
  pub grid_auto_rows: Vec<taffy::TrackSizingFunction>,
  pub grid_auto_columns: Vec<taffy::TrackSizingFunction>,
  pub grid_auto_flow: taffy::GridAutoFlow,
  pub grid_template_areas: Vec<taffy::GridTemplateArea<String>>,
  pub grid_template_column_names: Vec<Vec<String>>,
  pub grid_template_row_names: Vec<Vec<String>>,
  #[refine]
  pub grid_row: taffy::Line<taffy::GridPlacement<String>>,
  #[refine]
  pub grid_column: taffy::Line<taffy::GridPlacement<String>>,
}

impl Default for Style {
  fn default() -> Self {
    taffy::Style::DEFAULT.into()
  }
}

impl From<taffy::Style> for Style {
  fn from(style: taffy::Style) -> Self {
    Self {
      display: style.display,
      item_is_table: style.item_is_table,
      item_is_replaced: style.item_is_replaced,
      box_sizing: style.box_sizing,
      direction: style.direction,
      overflow: style.overflow,
      scrollbar_width: style.scrollbar_width,
      position: style.position,
      inset: style.inset,
      size: style.size,
      min_size: style.min_size,
      max_size: style.max_size,
      aspect_ratio: style.aspect_ratio,
      margin: style.margin,
      padding: style.padding,
      border: style.border,
      align_items: style.align_items,
      align_self: style.align_self,
      justify_items: style.justify_items,
      justify_self: style.justify_self,
      align_content: style.align_content,
      justify_content: style.justify_content,
      gap: style.gap,
      text_align: style.text_align,
      flex_direction: style.flex_direction,
      flex_wrap: style.flex_wrap,
      flex_basis: style.flex_basis,
      flex_grow: style.flex_grow,
      flex_shrink: style.flex_shrink,
      grid_template_rows: style.grid_template_rows,
      grid_template_columns: style.grid_template_columns,
      grid_auto_rows: style.grid_auto_rows,
      grid_auto_columns: style.grid_auto_columns,
      grid_auto_flow: style.grid_auto_flow,
      grid_template_areas: style.grid_template_areas,
      grid_template_column_names: style.grid_template_column_names,
      grid_template_row_names: style.grid_template_row_names,
      grid_row: style.grid_row,
      grid_column: style.grid_column,
    }
  }
}

impl From<Style> for taffy::Style {
  fn from(style: Style) -> Self {
    Self {
      display: style.display,
      item_is_table: style.item_is_table,
      item_is_replaced: style.item_is_replaced,
      box_sizing: style.box_sizing,
      direction: style.direction,
      overflow: style.overflow,
      scrollbar_width: style.scrollbar_width,
      position: style.position,
      inset: style.inset,
      size: style.size,
      min_size: style.min_size,
      max_size: style.max_size,
      aspect_ratio: style.aspect_ratio,
      margin: style.margin,
      padding: style.padding,
      border: style.border,
      align_items: style.align_items,
      align_self: style.align_self,
      justify_items: style.justify_items,
      justify_self: style.justify_self,
      align_content: style.align_content,
      justify_content: style.justify_content,
      gap: style.gap,
      text_align: style.text_align,
      flex_direction: style.flex_direction,
      flex_wrap: style.flex_wrap,
      flex_basis: style.flex_basis,
      flex_grow: style.flex_grow,
      flex_shrink: style.flex_shrink,
      grid_template_rows: style.grid_template_rows,
      grid_template_columns: style.grid_template_columns,
      grid_auto_rows: style.grid_auto_rows,
      grid_auto_columns: style.grid_auto_columns,
      grid_auto_flow: style.grid_auto_flow,
      grid_template_areas: style.grid_template_areas,
      grid_template_column_names: style.grid_template_column_names,
      grid_template_row_names: style.grid_template_row_names,
      grid_row: style.grid_row,
      grid_column: style.grid_column,
      ..Self::DEFAULT
    }
  }
}
