use crate::style::*;
pub use smol_str::SmolStr as StyleClass;

#[derive(Default)]
pub struct Theme {
    styles: rapidhash::RapidHashMap<StyleClass, Style>,
}

impl Theme {
    pub const UNIVERSAL_CLASS: StyleClass = StyleClass::new_static("");
    pub const ROOT_TYPE_CLASS: StyleClass = StyleClass::new_static("###root");

    pub fn insert_style(&mut self, class: StyleClass, style: &Style) {
        if let Some(existing_style) = self.styles.get_mut(&class) {
            *existing_style = style.or_else(&existing_style);
        } else {
            self.styles.insert(class, style.clone());
        }
    }

    pub fn build_style(
        &self,
        explicit_style: Option<&Style>,
        custom_classes: &[StyleClass],
        type_class: StyleClass,
    ) -> Style {
        let mut style = explicit_style.cloned().unwrap_or(Style::DEFAULT);

        for custom_class in custom_classes {
            if let Some(class_style) = self.styles.get(custom_class) {
                style = style.or_else(class_style);
            }
        }

        if let Some(class_style) = self.styles.get(&type_class) {
            style = style.or_else(class_style);
        }

        if let Some(class_style) = self.styles.get(&Self::UNIVERSAL_CLASS) {
            style = style.or_else(class_style);
        }

        style
    }

    pub fn build_style_property<T: Clone, const INHERIT_FALLBACK: bool>(
        &self,
        select_property: impl Fn(&Style) -> &Property<T, INHERIT_FALLBACK>,
        explicit_style: Option<&Style>,
        custom_classes: &[StyleClass],
        type_class: StyleClass,
    ) -> Property<T, INHERIT_FALLBACK> {
        let mut property = select_property(explicit_style.unwrap_or(&Style::DEFAULT)).clone();

        for custom_class in custom_classes {
            if let Some(class_style) = self.styles.get(custom_class) {
                property = property.or_else(select_property(class_style));
            }
        }

        if let Some(class_style) = self.styles.get(&type_class) {
            property = property.or_else(select_property(class_style));
        }

        if let Some(class_style) = self.styles.get(&Self::UNIVERSAL_CLASS) {
            property = property.or_else(select_property(class_style));
        }

        property
    }
}
