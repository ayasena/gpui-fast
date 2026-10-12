//! Where a line of Chinese or Japanese text may not be wrapped.
//!
//! [`crate::LineWrapper`] may break before any character that is not a word
//! character (CJK is not space separated), so a full-width comma or a closing
//! bracket can open a line of its own: `非常好` / `，我们一起`. CJK
//! typesetting forbids that (W3C clreq / jlreq, UAX #14 classes CL, CP, EX,
//! IS and NS), and equally forbids an opening bracket or quote ending a line.
//! The wrapper asks [`allows_break_between`] before it takes a break in front
//! of such a character, so the line breaks one character earlier and the
//! punctuation moves down together with the character it belongs to.

/// Whether a line may wrap between `prev` and `c`, two characters with no
/// space between them, in text that otherwise breaks between any two
/// characters.
pub(crate) fn allows_break_between(prev: char, c: char) -> bool {
    !never_starts_line(c) && !never_ends_line(prev) && !(prev == '—' && c == '—')
}

/// Closing and full-width punctuation that must stay at the end of the line
/// before it (clreq's 行首禁则).
fn never_starts_line(c: char) -> bool {
    matches!(
        c,
        // Full-width stops, commas and marks.
        '、' | '。' | '，' | '．' | '：' | '；' | '？' | '！' | '｡' | '､'
            // Closing brackets and quotes.
            | '）' | '］' | '｝' | '」' | '』' | '】' | '〉' | '》' | '〕' | '〗' | '〙'
            | '〛' | '〞' | '〟' | '｠' | '｣' | '’' | '”'
            // Ellipses, middle dots and the like, iteration marks.
            | '…' | '‥' | '・' | '･' | '〜' | '‼' | '⁇' | '⁈' | '⁉' | '々' | '〻' | 'ゝ'
            | 'ゞ' | 'ヽ' | 'ヾ'
    )
}

/// Opening brackets and quotes that must start the line after them
/// (clreq's 行尾禁则).
fn never_ends_line(c: char) -> bool {
    matches!(
        c,
        '（' | '［'
            | '｛'
            | '「'
            | '『'
            | '【'
            | '〈'
            | '《'
            | '〔'
            | '〖'
            | '〘'
            | '〚'
            | '〝'
            | '｟'
            | '｢'
            | '‘'
            | '“'
    )
}

#[cfg(test)]
mod tests {
    use super::{allows_break_between, never_ends_line, never_starts_line};
    use crate::{
        Boundary, IndentAdjustment, LineFragment, LineWrapper, Pixels, TestAppContext,
        TestDispatcher, TextSystem, font, px,
    };
    use std::sync::Arc;

    const SENTENCE: &str = "今天的天气非常好，我们一起去公园散步吧。";

    fn wrapper() -> (LineWrapper, Arc<TextSystem>) {
        let cx = TestAppContext::build(TestDispatcher::new(0), None);
        let text_system = cx.text_system().clone();
        let id = text_system.resolve_font(&font(".ZedMono"));
        (
            LineWrapper::new(id, px(16.), text_system.clone()),
            text_system,
        )
    }

    fn width_of(text_system: &TextSystem, wrapper: &LineWrapper, text: &str) -> Pixels {
        text.chars()
            .map(|c| {
                text_system
                    .advance(wrapper.font_id, wrapper.font_size, c)
                    .unwrap()
                    .width
            })
            .fold(px(0.), |sum, width| sum + width)
    }

    fn wrap(wrapper: &mut LineWrapper, text: &str, width: Pixels) -> Vec<usize> {
        wrapper
            .wrap_line(
                &[LineFragment::text(text)],
                width,
                IndentAdjustment::NoIndent,
            )
            .map(|Boundary { ix, .. }| ix)
            .collect()
    }

    #[test]
    fn a_comma_moves_down_with_the_character_before_it() {
        let (mut wrapper, text_system) = wrapper();
        let fits = "今天的天气非常好";
        let comma = width_of(&text_system, &wrapper, "，");
        assert!(comma > px(0.));
        // The line holds everything before the comma, but not the comma.
        let width = width_of(&text_system, &wrapper, fits) + comma / 2.;

        let boundaries = wrap(&mut wrapper, SENTENCE, width);
        assert_eq!(boundaries[0], fits.len() - "好".len());
        assert_eq!(&SENTENCE[boundaries[0]..], "好，我们一起去公园散步吧。");
    }

    #[test]
    fn no_width_starts_a_line_with_closing_or_ends_one_with_opening_punctuation() {
        let text = "我用（Rust）重写了程序：速度提升了８％，超出预期……「渲染」部分表现亮眼——尤其是《文档》提到的部分。";
        let (mut wrapper, text_system) = wrapper();
        let one = width_of(&text_system, &wrapper, "中");
        // From the narrowest line `（Rust）` fits on, so no break is forced.
        for columns in 6..text.chars().count() {
            let width = one * columns as f32 + one / 2.;
            for ix in wrap(&mut wrapper, text, width) {
                let next = text[ix..].chars().next().unwrap();
                let prev = text[..ix].chars().next_back().unwrap();
                assert!(
                    !never_starts_line(next) && !never_ends_line(prev),
                    "{columns} columns: `{}` / `{}`",
                    &text[..ix],
                    &text[ix..],
                );
            }
        }
    }

    #[test]
    fn ideographs_and_latin_text_still_break_as_before() {
        assert!(allows_break_between('好', '我'));
        assert!(allows_break_between('，', '我'));
        assert!(allows_break_between('用', '（'));
        assert!(allows_break_between('）', '重'));
        assert!(!allows_break_between('—', '—'));

        let (mut wrapper, text_system) = wrapper();
        let one = width_of(&text_system, &wrapper, "中");
        assert_eq!(wrap(&mut wrapper, "中文中文中文", one * 2.5), [6, 12]);

        // Latin words still break at spaces only, and CJK after them anywhere.
        let width = width_of(&text_system, &wrapper, "world, 你好") + one / 2.;
        assert_eq!(wrap(&mut wrapper, "Hello world, 你好世界", width), [6, 19]);
    }
}
