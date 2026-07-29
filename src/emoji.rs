use std::{collections::HashMap, ops::Range, sync::OnceLock};

use regex::Regex;
use serenity::all::ReactionType;

#[derive(Clone, Copy)]
pub(crate) struct CustomEmoji {
    pub(crate) name: &'static str,
    id: u64,
}

impl CustomEmoji {
    pub(crate) fn reaction(self) -> ReactionType {
        ReactionType::Custom {
            animated: false,
            id: self.id.into(),
            name: Some(self.name.to_owned()),
        }
    }

    pub(crate) fn mention(self) -> String {
        format!("<:{}:{}>", self.name, self.id)
    }
}

pub(crate) const VRISKA_EMOJI: CustomEmoji = CustomEmoji {
    name: "vriska",
    id: 1_017_263_376_361_062_490,
};
pub(crate) const THUMBSUPDIRK_EMOJI: CustomEmoji = CustomEmoji {
    name: "thumbsupdirk",
    id: 1_016_921_360_674_598_944,
};
const JOHNDAB_EMOJI: CustomEmoji = CustomEmoji {
    name: "johndab",
    id: 1_023_722_986_332_749_834,
};
const ROSEDAB_EMOJI: CustomEmoji = CustomEmoji {
    name: "rosedab",
    id: 1_023_722_984_680_214_528,
};
const DAVEDAB_EMOJI: CustomEmoji = CustomEmoji {
    name: "davedab",
    id: 1_023_722_989_298_122_824,
};
const JADEDAB_EMOJI: CustomEmoji = CustomEmoji {
    name: "jadedab",
    id: 1_023_722_987_834_331_156,
};

const EMOJI_TRIGGERS: &[(CustomEmoji, &[&str])] = &[
    (VRISKA_EMOJI, &["vriska", "serket"]),
    (JOHNDAB_EMOJI, &["john", "egbert"]),
    (ROSEDAB_EMOJI, &["rose", "lalonde"]),
    (DAVEDAB_EMOJI, &["dave", "strider"]),
    (JADEDAB_EMOJI, &["jade", "harley"]),
];

pub(crate) fn triggered_emojis(message: &str) -> Vec<CustomEmoji> {
    let casefolded = message.to_lowercase();
    let mut matches = EMOJI_TRIGGERS
        .iter()
        .filter_map(|(emoji, triggers)| {
            triggers
                .iter()
                .filter_map(|trigger| casefolded.find(trigger))
                .min()
                .map(|position| (*emoji, position))
        })
        .collect::<Vec<_>>();
    matches.sort_by_key(|(_, position)| *position);
    matches.into_iter().map(|(emoji, _)| emoji).collect()
}

pub(crate) fn ordered_poll_emojis(message: &str) -> Vec<ReactionType> {
    let mut positions: HashMap<String, (usize, ReactionType)> = HashMap::new();
    let custom_mentions = custom_emoji_mentions(message);

    for (range, emoji_mention) in &custom_mentions {
        if let Ok(reaction) = ReactionType::try_from(emoji_mention.as_str()) {
            positions
                .entry(emoji_mention.clone())
                .or_insert((range.start, reaction));
        }
    }

    for (position, emoji) in unicode_emojis(message) {
        if custom_mentions
            .iter()
            .any(|(range, _)| range.contains(&position))
        {
            continue;
        }

        positions
            .entry(emoji.clone())
            .or_insert((position, ReactionType::Unicode(emoji)));
    }

    let mut ordered = positions.into_values().collect::<Vec<_>>();
    ordered.sort_by_key(|(position, _)| *position);
    ordered.into_iter().map(|(_, emoji)| emoji).collect()
}

fn custom_emoji_mentions(message: &str) -> Vec<(Range<usize>, String)> {
    static CUSTOM_EMOJI_RE: OnceLock<Regex> = OnceLock::new();
    let custom_emoji_re =
        CUSTOM_EMOJI_RE.get_or_init(|| Regex::new(r"<a?:[A-Za-z0-9_]+:[0-9]+>").unwrap());

    custom_emoji_re
        .find_iter(message)
        .map(|found| (found.range(), found.as_str().to_owned()))
        .collect()
}

fn unicode_emojis(message: &str) -> Vec<(usize, String)> {
    static EMOJI_RE: OnceLock<Regex> = OnceLock::new();
    let emoji_re = EMOJI_RE.get_or_init(|| {
        Regex::new(
            r"(?:\p{Regional_Indicator}{2}|[0-9#*]\u{FE0F}?\u{20E3}|[\p{Emoji_Presentation}\p{Extended_Pictographic}]\u{FE0F}?\p{Emoji_Modifier}?(?:\u{200D}[\p{Emoji_Presentation}\p{Extended_Pictographic}]\u{FE0F}?\p{Emoji_Modifier}?)*)",
        )
        .expect("valid emoji regex")
    });

    emoji_re
        .find_iter(message)
        .map(|found| (found.start(), found.as_str().to_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unicode(reaction: &ReactionType) -> Option<&str> {
        match reaction {
            ReactionType::Unicode(value) => Some(value),
            _ => None,
        }
    }

    #[test]
    fn triggered_emojis_follow_first_trigger_position() {
        let names = triggered_emojis("rose talked to vriska and john")
            .into_iter()
            .map(|emoji| emoji.name)
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["rosedab", "vriska", "johndab"]);
    }

    #[test]
    fn poll_emojis_do_not_extract_digits_from_custom_emoji_ids() {
        let emojis = ordered_poll_emojis("Vote <:foo:123456789012345678> ✅");

        assert_eq!(emojis.len(), 2);
        assert!(matches!(emojis[0], ReactionType::Custom { .. }));
        assert_eq!(unicode(&emojis[1]), Some("✅"));
    }

    #[test]
    fn poll_emojis_keep_first_occurrence_and_remove_duplicates() {
        let emojis = ordered_poll_emojis("✅ <:foo:123456789012345678> ✅");

        assert_eq!(emojis.len(), 2);
        assert_eq!(unicode(&emojis[0]), Some("✅"));
        assert!(matches!(emojis[1], ReactionType::Custom { .. }));
    }

    #[test]
    fn poll_emojis_include_animated_custom_emoji_and_zwj_unicode() {
        let emojis = ordered_poll_emojis("<a:spin:123456789012345678> 👨‍👩‍👧‍👦");

        assert_eq!(emojis.len(), 2);
        assert!(matches!(
            emojis[0],
            ReactionType::Custom { animated: true, .. }
        ));
        assert_eq!(unicode(&emojis[1]), Some("👨‍👩‍👧‍👦"));
    }
}
