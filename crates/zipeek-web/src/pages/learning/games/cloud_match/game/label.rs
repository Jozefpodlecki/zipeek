use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LabelScale {
    Large,
    Medium,
    Compact,
}

fn label_scale(label: &str, primary: bool) -> LabelScale {
    let count = label.chars().count();
    match (primary, count) {
        (true, 0..=3) => LabelScale::Large,
        (true, 4..=5) => LabelScale::Medium,
        (false, 0..=10) => LabelScale::Large,
        _ => LabelScale::Compact,
    }
}

pub(super) fn compact_label(label: &str, primary: bool) -> String {
    if primary {
        if label.chars().count() <= 8 {
            return label.to_owned();
        }
        return format!("{}…", label.chars().take(7).collect::<String>());
    }

    let phrase = label.split([',', ';', '(']).next().unwrap_or(label).trim();
    let phrase = if phrase.is_empty() {
        label.trim()
    } else {
        phrase
    };
    if phrase.chars().count() <= 18 {
        return phrase.to_owned();
    }
    let mut short = String::new();
    for word in phrase.split_whitespace() {
        if short.chars().count() + word.chars().count() + usize::from(!short.is_empty()) > 16 {
            break;
        }
        if !short.is_empty() {
            short.push(' ');
        }
        short.push_str(word);
    }
    if short.is_empty() {
        short = phrase.chars().take(15).collect();
    }
    format!("{}…", short.trim_end())
}

pub(super) fn view(label: &str, primary: bool) -> Html {
    let visible = compact_label(label, primary);
    let (font_size, mobile_font_size) = match (primary, label_scale(&visible, primary)) {
        (true, LabelScale::Large) => ("text-[clamp(1.85rem,2.3vw,2.4rem)]", "max-sm:text-[1.6rem]"),
        (true, LabelScale::Medium) => (
            "text-[clamp(1.5rem,1.9vw,1.95rem)]",
            "max-sm:text-[1.35rem]",
        ),
        (true, LabelScale::Compact) => (
            "text-[clamp(1.15rem,1.45vw,1.5rem)]",
            "max-sm:text-[1.1rem]",
        ),
        (false, LabelScale::Compact) => ("text-[clamp(.86rem,1vw,1rem)]", "max-sm:text-[.86rem]"),
        (false, _) => ("text-[clamp(1rem,1.25vw,1.23rem)]", "max-sm:text-[.94rem]"),
    };
    let label_classes = classes!(
        "pointer-events-none",
        "absolute",
        "left-[21%]",
        "right-[21%]",
        "top-[57%]",
        "z-[1]",
        "max-h-[3.35rem]",
        "-translate-y-1/2",
        "overflow-hidden",
        "text-center",
        "font-bold",
        "leading-[1.16]",
        "text-balance",
        "[display:-webkit-box]",
        "[-webkit-box-orient:vertical]",
        "[-webkit-line-clamp:2]",
        "[overflow-wrap:anywhere]",
        "max-sm:left-[19%]",
        "max-sm:right-[19%]",
        "max-sm:max-h-[2.8rem]",
        "contrast-more:text-[#092a45]",
        font_size,
        mobile_font_size,
        if primary {
            "leading-[1.1]"
        } else {
            "tracking-[-.015em]"
        },
    );
    html! {
        <span data-cloud-label="" class={label_classes}>
            {visible}
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hanzi_and_meanings_fit_their_intended_text_budgets() {
        assert_eq!(compact_label("牛奶", true), "牛奶");
        assert_eq!(compact_label("中华人民共和国", true), "中华人民共和…");
        assert_eq!(label_scale("中华人民共和国", true), LabelScale::Compact);
        assert_eq!(label_scale("牛奶", true), LabelScale::Large);
        assert!(
            compact_label("to get on or into (a train)", false)
                .chars()
                .count()
                <= 18
        );
        assert_eq!(compact_label("cow's milk", false), "cow's milk");
    }
}
