pub const RIBBI_STYLE_NAMES: [&str; 5] = ["Regular", "Italic", "Bold", "BoldItalic", "Bold Italic"];
pub const STATIC_STYLE_NAMES: [&str; 18] = [
    "Thin",
    "ExtraLight",
    "Light",
    "Regular",
    "Medium",
    "SemiBold",
    "Bold",
    "ExtraBold",
    "Black",
    "Thin Italic",
    "ExtraLight Italic",
    "Light Italic",
    "Italic",
    "Medium Italic",
    "SemiBold Italic",
    "Bold Italic",
    "ExtraBold Italic",
    "Black Italic",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphClass {
    Base,
    Ligature,
    Mark,
    Component,
}
impl GlyphClass {
    pub fn from_u16(class: u16) -> Option<Self> {
        match class {
            1 => Some(Self::Base),
            2 => Some(Self::Ligature),
            3 => Some(Self::Mark),
            4 => Some(Self::Component),
            _ => None,
        }
    }
}
