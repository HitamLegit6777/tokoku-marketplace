// Theme catalog. Each theme is a full palette + typography exposed to templates
// as CSS custom properties, so the storefront can be re-skinned with zero code.
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Theme {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    // colors
    pub primary: &'static str,
    pub primary_dark: &'static str,
    pub primary_light: &'static str,
    pub secondary: &'static str,
    pub accent: &'static str,
    pub bg: &'static str,
    pub bg_alt: &'static str,
    pub surface: &'static str,
    pub text: &'static str,
    pub text_muted: &'static str,
    pub border: &'static str,
    pub success: &'static str,
    pub danger: &'static str,
    pub on_primary: &'static str,
    // typography
    pub font_heading: &'static str,
    pub font_body: &'static str,
    pub google_fonts: &'static str,
    // shape
    pub radius: &'static str,
    pub radius_lg: &'static str,
    pub shadow: &'static str,
    pub hero_gradient: &'static str,
    pub btn_gradient: &'static str,
}

impl Theme {
    /// Emit the theme as a CSS custom-property block for :root.
    pub fn to_css_vars(&self) -> String {
        format!(
":root{{--primary:{primary};--primary-dark:{primary_dark};--primary-light:{primary_light};\
--secondary:{secondary};--accent:{accent};--bg:{bg};--bg-alt:{bg_alt};--surface:{surface};\
--text:{text};--text-muted:{text_muted};--border:{border};--success:{success};--danger:{danger};\
--on-primary:{on_primary};--font-heading:{font_heading};--font-body:{font_body};\
--radius:{radius};--radius-lg:{radius_lg};--shadow:{shadow};--hero-gradient:{hero_gradient};\
--btn-gradient:{btn_gradient};}}",
            primary = self.primary, primary_dark = self.primary_dark, primary_light = self.primary_light,
            secondary = self.secondary, accent = self.accent, bg = self.bg, bg_alt = self.bg_alt,
            surface = self.surface, text = self.text, text_muted = self.text_muted, border = self.border,
            success = self.success, danger = self.danger, on_primary = self.on_primary,
            font_heading = self.font_heading, font_body = self.font_body, radius = self.radius,
            radius_lg = self.radius_lg, shadow = self.shadow, hero_gradient = self.hero_gradient,
            btn_gradient = self.btn_gradient,
        )
    }
}

/// All available themes. Order here is the order shown in the theme picker.
pub const THEMES: &[Theme] = &[
    Theme {
        id: "sunset",
        name: "Sunset Glow",
        description: "Hangat & ramah, cocok untuk kuliner dan fashion",
        primary: "#ff6b35", primary_dark: "#e8551f", primary_light: "#ff8c5f",
        secondary: "#004e89", accent: "#ffc93c",
        bg: "#fffaf5", bg_alt: "#fff1e6", surface: "#ffffff",
        text: "#2b2118", text_muted: "#7a6f66", border: "#f0e2d5",
        success: "#2e9e5b", danger: "#e23b3b", on_primary: "#ffffff",
        font_heading: "'Poppins',sans-serif", font_body: "'Inter',sans-serif",
        google_fonts: "family=Poppins:wght@500;600;700;800&family=Inter:wght@400;500;600",
        radius: "12px", radius_lg: "24px",
        shadow: "0 10px 30px rgba(255,107,53,.12)",
        hero_gradient: "linear-gradient(135deg,#ff6b35 0%,#ffc93c 100%)",
        btn_gradient: "linear-gradient(135deg,#ff6b35,#e8551f)",
    },
    Theme {
        id: "ocean",
        name: "Ocean Breeze",
        description: "Bersih & profesional, cocok untuk elektronik dan jasa",
        primary: "#0ea5e9", primary_dark: "#0284c7", primary_light: "#38bdf8",
        secondary: "#1e293b", accent: "#06b6d4",
        bg: "#f8fafc", bg_alt: "#eff6ff", surface: "#ffffff",
        text: "#0f172a", text_muted: "#64748b", border: "#e2e8f0",
        success: "#10b981", danger: "#ef4444", on_primary: "#ffffff",
        font_heading: "'Plus Jakarta Sans',sans-serif", font_body: "'Inter',sans-serif",
        google_fonts: "family=Plus+Jakarta+Sans:wght@500;600;700;800&family=Inter:wght@400;500;600",
        radius: "10px", radius_lg: "20px",
        shadow: "0 10px 30px rgba(14,165,233,.12)",
        hero_gradient: "linear-gradient(135deg,#0ea5e9 0%,#06b6d4 100%)",
        btn_gradient: "linear-gradient(135deg,#0ea5e9,#0284c7)",
    },
    Theme {
        id: "forest",
        name: "Forest Fresh",
        description: "Natural & organik, cocok untuk produk sehat dan herbal",
        primary: "#16a34a", primary_dark: "#15803d", primary_light: "#4ade80",
        secondary: "#365314", accent: "#84cc16",
        bg: "#f7fdf9", bg_alt: "#ecfdf5", surface: "#ffffff",
        text: "#14261a", text_muted: "#5f7167", border: "#dcefe2",
        success: "#16a34a", danger: "#dc2626", on_primary: "#ffffff",
        font_heading: "'Fraunces',serif", font_body: "'Nunito Sans',sans-serif",
        google_fonts: "family=Fraunces:opsz,wght@9..144,500;9..144,600;9..144,700&family=Nunito+Sans:wght@400;600;700",
        radius: "14px", radius_lg: "28px",
        shadow: "0 10px 30px rgba(22,163,74,.12)",
        hero_gradient: "linear-gradient(135deg,#16a34a 0%,#84cc16 100%)",
        btn_gradient: "linear-gradient(135deg,#16a34a,#15803d)",
    },
    Theme {
        id: "berry",
        name: "Berry Bloom",
        description: "Feminin & elegan, cocok untuk kecantikan dan aksesoris",
        primary: "#db2777", primary_dark: "#be185d", primary_light: "#f472b6",
        secondary: "#701a75", accent: "#a855f7",
        bg: "#fff7fb", bg_alt: "#fdf2f8", surface: "#ffffff",
        text: "#2d1524", text_muted: "#8b6478", border: "#f6e0ec",
        success: "#059669", danger: "#e11d48", on_primary: "#ffffff",
        font_heading: "'Playfair Display',serif", font_body: "'Poppins',sans-serif",
        google_fonts: "family=Playfair+Display:wght@500;600;700;800&family=Poppins:wght@400;500;600",
        radius: "16px", radius_lg: "30px",
        shadow: "0 10px 30px rgba(219,39,119,.14)",
        hero_gradient: "linear-gradient(135deg,#db2777 0%,#a855f7 100%)",
        btn_gradient: "linear-gradient(135deg,#db2777,#be185d)",
    },
    Theme {
        id: "midnight",
        name: "Midnight Luxe",
        description: "Mewah & premium dengan nuansa gelap, cocok untuk brand eksklusif",
        primary: "#f59e0b", primary_dark: "#d97706", primary_light: "#fbbf24",
        secondary: "#fbbf24", accent: "#f59e0b",
        bg: "#0f1117", bg_alt: "#171923", surface: "#1c1f2b",
        text: "#f3f4f6", text_muted: "#9ca3af", border: "#2b2f3d",
        success: "#34d399", danger: "#f87171", on_primary: "#1a1200",
        font_heading: "'Cormorant Garamond',serif", font_body: "'Jost',sans-serif",
        google_fonts: "family=Cormorant+Garamond:wght@500;600;700&family=Jost:wght@400;500;600",
        radius: "8px", radius_lg: "18px",
        shadow: "0 12px 40px rgba(0,0,0,.5)",
        hero_gradient: "linear-gradient(135deg,#1c1f2b 0%,#0f1117 100%)",
        btn_gradient: "linear-gradient(135deg,#f59e0b,#d97706)",
    },
    Theme {
        id: "mono",
        name: "Mono Minimal",
        description: "Minimalis hitam-putih, fokus pada produk (gaya butik modern)",
        primary: "#111111", primary_dark: "#000000", primary_light: "#333333",
        secondary: "#111111", accent: "#f43f5e",
        bg: "#ffffff", bg_alt: "#f5f5f5", surface: "#ffffff",
        text: "#111111", text_muted: "#737373", border: "#e5e5e5",
        success: "#059669", danger: "#dc2626", on_primary: "#ffffff",
        font_heading: "'Space Grotesk',sans-serif", font_body: "'Inter',sans-serif",
        google_fonts: "family=Space+Grotesk:wght@500;600;700&family=Inter:wght@400;500;600",
        radius: "4px", radius_lg: "8px",
        shadow: "0 6px 24px rgba(0,0,0,.08)",
        hero_gradient: "linear-gradient(135deg,#111111 0%,#404040 100%)",
        btn_gradient: "linear-gradient(135deg,#111111,#000000)",
    },
    Theme {
        id: "candy",
        name: "Candy Pop",
        description: "Ceria & playful, cocok untuk produk anak dan mainan",
        primary: "#8b5cf6", primary_dark: "#7c3aed", primary_light: "#a78bfa",
        secondary: "#ec4899", accent: "#22d3ee",
        bg: "#fbfaff", bg_alt: "#f3f0ff", surface: "#ffffff",
        text: "#1e1b2e", text_muted: "#6b6786", border: "#ebe5fb",
        success: "#10b981", danger: "#f43f5e", on_primary: "#ffffff",
        font_heading: "'Baloo 2',cursive", font_body: "'Quicksand',sans-serif",
        google_fonts: "family=Baloo+2:wght@500;600;700;800&family=Quicksand:wght@400;500;600;700",
        radius: "20px", radius_lg: "34px",
        shadow: "0 12px 34px rgba(139,92,246,.16)",
        hero_gradient: "linear-gradient(135deg,#8b5cf6 0%,#ec4899 60%,#22d3ee 120%)",
        btn_gradient: "linear-gradient(135deg,#8b5cf6,#7c3aed)",
    },
    Theme {
        id: "coffee",
        name: "Coffee House",
        description: "Cozy & artisan, cocok untuk kopi, roti, dan produk handmade",
        primary: "#a16207", primary_dark: "#854d0e", primary_light: "#ca8a04",
        secondary: "#44403c", accent: "#b45309",
        bg: "#faf7f2", bg_alt: "#f5efe6", surface: "#fffdf9",
        text: "#292018", text_muted: "#78716c", border: "#e7dccb",
        success: "#4d7c0f", danger: "#b91c1c", on_primary: "#ffffff",
        font_heading: "'DM Serif Display',serif", font_body: "'DM Sans',sans-serif",
        google_fonts: "family=DM+Serif+Display:ital@0;1&family=DM+Sans:wght@400;500;700",
        radius: "10px", radius_lg: "22px",
        shadow: "0 10px 30px rgba(161,98,7,.13)",
        hero_gradient: "linear-gradient(135deg,#a16207 0%,#b45309 100%)",
        btn_gradient: "linear-gradient(135deg,#a16207,#854d0e)",
    },
];

/// Look up a theme by id, falling back to the first theme.
pub fn get_theme(id: &str) -> &'static Theme {
    THEMES.iter().find(|t| t.id == id).unwrap_or(&THEMES[0])
}

/// All Google Fonts families concatenated for a single preload URL (dedup not needed).
pub fn all_google_fonts() -> String {
    THEMES
        .iter()
        .map(|t| t.google_fonts)
        .collect::<Vec<_>>()
        .join("&")
}
