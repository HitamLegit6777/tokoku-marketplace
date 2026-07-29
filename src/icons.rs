//! Custom hand-authored line-icon set (stroke SVG, 24x24, currentColor).
//! No external icon library. Rendered via the `icon` Askama filter as
//! `{{ "cart"|icon|safe }}`. Unknown names fall back to a neutral dot so the
//! UI never breaks. Keep the visual language consistent: 1.75 stroke width,
//! round caps/joins, no fills (except intentional dots).

/// Return inline SVG markup for `name`, or a fallback glyph.
pub fn svg(name: &str) -> String {
    let body = inner(name);
    format!(
        "<svg class=\"ic ic-{n}\" viewBox=\"0 0 24 24\" fill=\"none\" \
stroke=\"currentColor\" stroke-width=\"1.75\" stroke-linecap=\"round\" \
stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">{b}</svg>",
        n = name,
        b = body
    )
}

/// Inner path markup per icon name.
fn inner(name: &str) -> &'static str {
    match name {
        // ---- shop / commerce ----
        "cart" => "<circle cx='9' cy='20' r='1.4'/><circle cx='18' cy='20' r='1.4'/><path d='M2.5 3h2.2l2 12.2a1.6 1.6 0 0 0 1.6 1.3h8.4a1.6 1.6 0 0 0 1.6-1.3L21 7H6'/>",
        "bag" => "<path d='M6 8h12l-1 12H7L6 8z'/><path d='M9 8V6a3 3 0 0 1 6 0v2'/>",
        "box" => "<path d='M21 8.5 12 13 3 8.5 12 4l9 4.5z'/><path d='M3 8.5V16l9 4.5 9-4.5V8.5'/><path d='M12 13v7.5'/>",
        "package" => "<path d='M21 8.5 12 13 3 8.5 12 4l9 4.5z'/><path d='M3 8.5V16l9 4.5 9-4.5V8.5'/><path d='M12 13v7.5'/><path d='M7.5 6.2 16.5 10.8'/>",
        "tag" => "<path d='M20 12.5 12.5 20 4 11.5V4h7.5L20 12.5z'/><circle cx='8.5' cy='8.5' r='1.2'/>",
        "ticket" => "<path d='M4 8a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v2a2 2 0 0 0 0 4v2a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-2a2 2 0 0 0 0-4V8z'/><path d='M14 6v12' stroke-dasharray='2 2'/>",
        "gift" => "<rect x='4' y='9' width='16' height='11' rx='1.5'/><path d='M4 13h16'/><path d='M12 9v11'/><path d='M12 9C10 9 8 8 8 6.5A2 2 0 0 1 12 6a2 2 0 0 1 4 .5C16 8 14 9 12 9z'/>",
        _ => inner2(name),
    }
}

fn inner2(name: &str) -> &'static str {
    match name {
        // ---- food / category ----
        "coffee" => "<path d='M4 8h13v4a5 5 0 0 1-5 5H9a5 5 0 0 1-5-5V8z'/><path d='M17 9h2a2 2 0 0 1 0 4h-2'/><path d='M7 3v2M11 3v2'/>",
        "cup" => "<path d='M5 4h11l-1 15a2 2 0 0 1-2 1.8H8A2 2 0 0 1 6 19L5 4z'/><path d='M6 9h9'/>",
        "cookie" => "<path d='M12 3a9 9 0 1 0 9 9 3 3 0 0 1-3-3 3 3 0 0 1-3-3 3 3 0 0 1-3-3z'/><circle cx='9' cy='11' r='.9'/><circle cx='13' cy='14' r='.9'/><circle cx='15' cy='9' r='.9'/>",
        // ---- ui / nav ----
        "search" => "<circle cx='11' cy='11' r='6.5'/><path d='M20 20l-4-4'/>",
        "menu" => "<path d='M4 7h16M4 12h16M4 17h16'/>",
        "link" => "<path d='M10 13a4 4 0 0 0 6 .5l2-2a4 4 0 0 0-5.7-5.7l-1 1'/><path d='M14 11a4 4 0 0 0-6-.5l-2 2A4 4 0 0 0 11.7 18l1-1'/>",
        "external" => "<path d='M14 4h6v6'/><path d='M20 4l-9 9'/><path d='M18 14v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4'/>",
        "settings" => "<circle cx='12' cy='12' r='3'/><path d='M12 2v2.5M12 19.5V22M4.2 4.2l1.8 1.8M18 18l1.8 1.8M2 12h2.5M19.5 12H22M4.2 19.8 6 18M18 6l1.8-1.8'/>",
        "palette" => "<path d='M12 3a9 9 0 1 0 0 18 2 2 0 0 0 2-2 2 2 0 0 1 2-2h1.5A3.5 3.5 0 0 0 21 11.5 8.5 8.5 0 0 0 12 3z'/><circle cx='7.5' cy='12' r='1'/><circle cx='10' cy='8' r='1'/><circle cx='15' cy='8' r='1'/>",
        "chart" => "<path d='M4 4v16h16'/><path d='M8 15v-4M12 15V8M16 15v-6'/>",
        "receipt" => "<path d='M6 3h12v18l-2-1.3-2 1.3-2-1.3-2 1.3-2-1.3L6 21V3z'/><path d='M9 8h6M9 12h6'/>",
        "users" => "<circle cx='9' cy='8' r='3'/><path d='M3.5 20a5.5 5.5 0 0 1 11 0'/><path d='M16 5.2A3 3 0 0 1 16 11'/><path d='M17 14.2A5.5 5.5 0 0 1 20.5 20'/>",
        "money" => "<rect x='3' y='6' width='18' height='12' rx='2'/><circle cx='12' cy='12' r='2.5'/><path d='M6 9v6M18 9v6'/>",
        "page" => "<path d='M7 3h7l4 4v14H7V3z'/><path d='M14 3v4h4'/><path d='M9 12h6M9 16h6'/>",
        _ => inner3(name),
    }
}

fn inner3(name: &str) -> &'static str {
    match name {
        // ---- payment ----
        "bank" => "<path d='M12 3 3 8h18L12 3z'/><path d='M5 8v9M9 8v9M15 8v9M19 8v9'/><path d='M3 20h18'/>",
        "card" => "<rect x='3' y='5' width='18' height='14' rx='2'/><path d='M3 9h18'/><path d='M6 14h4'/>",
        "wallet" => "<path d='M4 7a2 2 0 0 1 2-2h11a2 2 0 0 1 2 2v1'/><path d='M4 7v10a2 2 0 0 0 2 2h13a1 1 0 0 0 1-1v-8a1 1 0 0 0-1-1H6'/><circle cx='16.5' cy='12.5' r='1.2'/>",
        "qr" => "<rect x='4' y='4' width='6' height='6' rx='1'/><rect x='14' y='4' width='6' height='6' rx='1'/><rect x='4' y='14' width='6' height='6' rx='1'/><path d='M14 14h3v3M20 14v6M14 20h3'/>",
        "phone" => "<rect x='7' y='3' width='10' height='18' rx='2'/><path d='M11 18h2'/>",
        "cash" => "<rect x='3' y='6' width='18' height='12' rx='2'/><circle cx='12' cy='12' r='2.5'/><path d='M7 12h.01M17 12h.01'/>",
        "globe" => "<circle cx='12' cy='12' r='9'/><path d='M3 12h18'/><path d='M12 3a14 14 0 0 1 0 18 14 14 0 0 1 0-18z'/>",
        // ---- status / trust ----
        "check" => "<path d='M5 12.5 10 17.5 19 7'/>",
        "check-circle" => "<circle cx='12' cy='12' r='9'/><path d='M8 12.2 11 15.2 16 9'/>",
        "shield" => "<path d='M12 3 5 6v5c0 4.4 3 7.7 7 9 4-1.3 7-4.6 7-9V6l-7-3z'/><path d='M9 12l2 2 4-4'/>",
        "bolt" => "<path d='M13 3 5 13h5l-1 8 8-10h-5l1-8z'/>",
        "truck" => "<path d='M3 6h11v9H3z'/><path d='M14 9h4l3 3v3h-7z'/><circle cx='7' cy='18' r='1.6'/><circle cx='17.5' cy='18' r='1.6'/>",
        "chat" => "<path d='M4 5h16v11H9l-4 3v-3H4V5z'/><path d='M8 9h8M8 12h5'/>",
        "clip" => "<path d='M9 8v9a3 3 0 0 0 6 0V7a2 2 0 0 0-4 0v9a1 1 0 0 0 2 0V8'/>",
        "clipboard" => "<rect x='6' y='4' width='12' height='17' rx='2'/><path d='M9 4V3h6v1'/><path d='M9 10h6M9 14h4'/>",
        "warning" => "<path d='M12 4 2.5 20h19L12 4z'/><path d='M12 10v4'/><path d='M12 17h.01'/>",
        "help" => "<circle cx='12' cy='12' r='9'/><path d='M9.2 9.2a2.8 2.8 0 0 1 5.3 1c0 1.8-2.5 2.2-2.5 3.6'/><path d='M12 17h.01'/>",
        "star" => "<path d='M12 3.5 14.6 9l6 .8-4.4 4.2 1.1 6-5.3-2.9L6.3 20l1.1-6L3 9.8 9 9l3-5.5z'/>",
        "camera" => "<path d='M5 8h3l1.2-2h5.6L16 8h3a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V9a1 1 0 0 1 1-1z'/><circle cx='12' cy='13' r='3'/>",
        "image" => "<rect x='3' y='5' width='18' height='14' rx='2'/><circle cx='8.5' cy='10' r='1.6'/><path d='m4 17 4.5-4.5 3 3L15 12l5 5'/>",
        "arrow-right" => "<path d='M4 12h15'/><path d='M13 6l6 6-6 6'/>",
        "pencil" => "<path d='M4 20h4L18.5 9.5a2.1 2.1 0 0 0-3-3L5 17v3z'/><path d='M14.5 6.5l3 3'/>",
        _ => "<circle cx='12' cy='12' r='2.2' fill='currentColor' stroke='none'/>",
    }
}
