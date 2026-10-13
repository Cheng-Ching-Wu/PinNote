//! Pure sync decisions: no files, no database. Everything here is unit tested.

use std::collections::{BTreeMap, HashSet};

use crate::models::note::{normalize_tags, tag_key};
use crate::models::SyncedNote;

/// Result of merging one note that exists on both sides.
#[derive(Debug, Clone, PartialEq)]
pub struct Merged {
    pub note: SyncedNote,
    /// Title / content were changed on both sides: the losing text (kept as a revision).
    pub text_loser: Option<(String, String)>,
}

/// Field-by-field three-way merge. A field changed on one side only takes that
/// side; changed differently on both sides, the more recently modified note wins.
/// Title + content form one field so a note's text is never stitched together.
pub fn merge_note(base: Option<&SyncedNote>, local: &SyncedNote, remote: &SyncedNote) -> Merged {
    // Same timestamp: fall back to the text so every device picks the same winner.
    let local_wins = (&local.modified_at, &local.content, &local.title)
        >= (&remote.modified_at, &remote.content, &remote.title);

    let text = |n: &SyncedNote| (n.title.clone(), n.content.clone());
    let (title_content, text_conflict) =
        pick(base.map(text), text(local), text(remote), local_wins);
    let (color, _) = pick(base.map(|b| b.color), local.color, remote.color, local_wins);
    let status = |n: &SyncedNote| (n.status, n.deleted_at.clone());
    let ((status, deleted_at), _) =
        pick(base.map(status), status(local), status(remote), local_wins);
    let (sort_order, _) = pick(
        base.map(|b| b.sort_order),
        local.sort_order,
        remote.sort_order,
        local_wins,
    );
    let tags = merge_tags(base.map(|b| b.tags.as_slice()), &local.tags, &remote.tags);

    let text_loser = text_conflict.then(|| {
        if local_wins {
            text(remote)
        } else {
            text(local)
        }
    });
    let (title, content) = title_content;
    Merged {
        note: SyncedNote {
            id: local.id.clone(),
            title,
            content,
            color,
            tags,
            status,
            deleted_at,
            sort_order,
            created_at: local.created_at.clone().min(remote.created_at.clone()),
            updated_at: local.updated_at.clone().max(remote.updated_at.clone()),
            modified_at: local.modified_at.clone().max(remote.modified_at.clone()),
        },
        text_loser,
    }
}

/// One field of a three-way merge. Returns the value and whether it was a real conflict.
fn pick<T: PartialEq>(base: Option<T>, local: T, remote: T, local_wins: bool) -> (T, bool) {
    if local == remote {
        return (local, false);
    }
    match base {
        Some(base) if base == local => (remote, false),
        Some(base) if base == remote => (local, false),
        _ if local_wins => (local, true),
        _ => (remote, true),
    }
}

/// Tags merge as a set: added on either side → kept, removed on either side → removed.
/// Without a common ancestor (first sync) both sides' tags are kept.
pub fn merge_tags(base: Option<&[String]>, local: &[String], remote: &[String]) -> Vec<String> {
    if local == remote {
        return local.to_vec();
    }
    let keys = |tags: &[String]| tags.iter().map(|t| tag_key(t)).collect::<HashSet<_>>();
    let (local_keys, remote_keys) = (keys(local), keys(remote));
    let removed: HashSet<String> = base
        .map(|base| {
            keys(base)
                .into_iter()
                .filter(|key| !local_keys.contains(key) || !remote_keys.contains(key))
                .collect()
        })
        .unwrap_or_default();
    let merged: Vec<String> = local
        .iter()
        .chain(remote)
        .filter(|tag| !removed.contains(&tag_key(tag)))
        .cloned()
        .collect();
    normalize_tags(&merged)
}

/// Several versions of one note in the sync folder (the file plus conflict copies
/// made by the cloud drive). The newest wins; the texts of the others are returned
/// so they can be kept as revisions.
pub fn resolve_versions(
    mut versions: Vec<SyncedNote>,
) -> Option<(SyncedNote, Vec<(String, String)>)> {
    versions.sort_by(|a, b| {
        (&b.modified_at, &b.content, &b.title).cmp(&(&a.modified_at, &a.content, &a.title))
    });
    let mut versions = versions.into_iter();
    let winner = versions.next()?;
    let losers = versions
        .filter(|v| (&v.title, &v.content) != (&winner.title, &winner.content))
        .map(|v| (v.title, v.content))
        .collect();
    Some((winner, losers))
}

/// What happens to one note on this device.
// One per note and run: the size difference doesn't matter.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum LocalAction {
    #[default]
    None,
    /// Insert or overwrite the synced fields.
    Upsert(SyncedNote),
    /// Delete for good (it was deleted on another device).
    Delete,
}

/// What happens to one note in the sync folder.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum RemoteAction {
    #[default]
    None,
    Write(SyncedNote),
    /// Remove `notes/<id>.json`, writing `deleted/<id>.json` with this time if given.
    Delete {
        tombstone: Option<String>,
    },
    /// Only write `deleted/<id>.json` (there was no note file).
    Tombstone(String),
}

// One per note and run: the size difference doesn't matter.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum BaseAction {
    #[default]
    Keep,
    Set(SyncedNote),
    Clear,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NotePlan {
    pub local: LocalAction,
    pub remote: RemoteAction,
    pub base: BaseAction,
    pub clear_local_tombstone: bool,
    /// Losing text of a conflict, kept as a revision of the note.
    pub revision: Option<(String, String)>,
    /// The state both sides agree on afterwards (None when the note is gone).
    pub merged: Option<SyncedNote>,
}

/// Everything known about one note id.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoteSides<'a> {
    pub local: Option<&'a SyncedNote>,
    /// When it was deleted for good on this device (not yet in the folder).
    pub local_tombstone: Option<&'a str>,
    pub remote: Option<&'a SyncedNote>,
    pub remote_tombstone: Option<&'a str>,
    /// State after the last successful sync.
    pub base: Option<&'a SyncedNote>,
}

/// Decides what to do with one note. A deletion only wins over edits made before
/// it; a note edited after being deleted elsewhere comes back.
pub fn plan_note(sides: NoteSides) -> NotePlan {
    let NoteSides {
        local,
        local_tombstone,
        remote,
        remote_tombstone,
        base,
    } = sides;
    let deleted_after =
        |note: &SyncedNote| remote_tombstone.is_some_and(|t| t > note.modified_at.as_str());
    let mut plan = NotePlan {
        // A local note with the same id (e.g. re-imported) supersedes an old local deletion.
        clear_local_tombstone: local.is_some() && local_tombstone.is_some(),
        ..NotePlan::default()
    };

    match (local, remote) {
        (Some(local), Some(remote)) => {
            if deleted_after(local) && deleted_after(remote) {
                plan.local = LocalAction::Delete;
                plan.remote = RemoteAction::Delete { tombstone: None };
                plan.base = BaseAction::Clear;
                return plan;
            }
            let merged = merge_note(base, local, remote);
            if merged.note != *local {
                plan.local = LocalAction::Upsert(merged.note.clone());
            }
            if merged.note != *remote {
                plan.remote = RemoteAction::Write(merged.note.clone());
            }
            plan.revision = merged.text_loser;
            plan.base = set_base(base, &merged.note);
            plan.merged = Some(merged.note);
        }
        (Some(local), None) => {
            if deleted_after(local) {
                plan.local = LocalAction::Delete;
                plan.base = BaseAction::Clear;
            } else {
                // New here, or the file went missing: (re)upload — never lose a local note.
                plan.remote = RemoteAction::Write(local.clone());
                plan.base = set_base(base, local);
                plan.merged = Some(local.clone());
            }
        }
        (None, Some(remote)) => {
            if deleted_after(remote) {
                plan.remote = RemoteAction::Delete { tombstone: None };
                plan.base = BaseAction::Clear;
            } else if let Some(deleted_at) = local_tombstone {
                plan.clear_local_tombstone = true;
                if remote.modified_at.as_str() > deleted_at {
                    // Edited elsewhere after we deleted it: bring it back.
                    plan.local = LocalAction::Upsert(remote.clone());
                    plan.base = set_base(base, remote);
                    plan.merged = Some(remote.clone());
                } else {
                    plan.remote = RemoteAction::Delete {
                        tombstone: Some(deleted_at.to_string()),
                    };
                    plan.base = BaseAction::Clear;
                }
            } else {
                plan.local = LocalAction::Upsert(remote.clone());
                plan.base = set_base(base, remote);
                plan.merged = Some(remote.clone());
            }
        }
        (None, None) => {
            if let Some(deleted_at) = local_tombstone {
                plan.clear_local_tombstone = true;
                if remote_tombstone.is_none() {
                    plan.remote = RemoteAction::Tombstone(deleted_at.to_string());
                }
            }
            if base.is_some() {
                plan.base = BaseAction::Clear;
            }
        }
    }
    plan
}

fn set_base(base: Option<&SyncedNote>, merged: &SyncedNote) -> BaseAction {
    if base == Some(merged) {
        BaseAction::Keep
    } else {
        BaseAction::Set(merged.clone())
    }
}

/// Three-way merge of a key → value map (tag colors). On a real conflict the local value wins.
pub fn merge_map(
    base: &BTreeMap<String, String>,
    local: &BTreeMap<String, String>,
    remote: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let keys: HashSet<&String> = base
        .keys()
        .chain(local.keys())
        .chain(remote.keys())
        .collect();
    keys.into_iter()
        .filter_map(|key| {
            let (b, l, r) = (base.get(key), local.get(key), remote.get(key));
            let value = if l == b { r } else { l };
            value.map(|v| (key.clone(), v.clone()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{NoteColor, NoteStatus};

    fn note(content: &str, modified_at: &str) -> SyncedNote {
        SyncedNote {
            id: "n1".into(),
            title: String::new(),
            content: content.into(),
            color: NoteColor::Yellow,
            tags: vec![],
            status: NoteStatus::Active,
            deleted_at: None,
            sort_order: 0,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: modified_at.into(),
            modified_at: modified_at.into(),
        }
    }

    const T1: &str = "2026-01-01T01:00:00.000Z";
    const T2: &str = "2026-01-01T02:00:00.000Z";
    const T3: &str = "2026-01-01T03:00:00.000Z";

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn one_sided_changes_merge_per_field() {
        let base = note("text", T1);
        let mut local = base.clone();
        local.color = NoteColor::Blue;
        local.modified_at = T3.into();
        let mut remote = base.clone();
        remote.content = "edited elsewhere".into();
        remote.modified_at = T2.into();

        let merged = merge_note(Some(&base), &local, &remote);
        assert_eq!(merged.note.content, "edited elsewhere");
        assert_eq!(merged.note.color, NoteColor::Blue);
        assert_eq!(merged.note.modified_at, T3);
        assert_eq!(merged.text_loser, None);
    }

    #[test]
    fn text_conflict_keeps_the_newer_and_reports_the_older() {
        let base = note("text", T1);
        let local = note("mine", T2);
        let remote = note("theirs", T3);
        let merged = merge_note(Some(&base), &local, &remote);
        assert_eq!(merged.note.content, "theirs");
        assert_eq!(merged.text_loser, Some((String::new(), "mine".into())));

        // Both devices reach the same result.
        let other_way = merge_note(Some(&base), &remote, &local);
        assert_eq!(other_way.note.content, "theirs");
    }

    #[test]
    fn tags_merge_as_sets() {
        let base = strings(&["a", "b"]);
        assert_eq!(
            merge_tags(Some(&base), &strings(&["a", "b", "c"]), &strings(&["b"])),
            strings(&["b", "c"])
        );
        assert_eq!(
            merge_tags(None, &strings(&["a"]), &strings(&["B", "A"])),
            strings(&["a", "B"])
        );
    }

    #[test]
    fn duplicate_files_resolve_to_the_newest() {
        let (winner, losers) =
            resolve_versions(vec![note("old", T1), note("new", T3), note("new", T2)]).unwrap();
        assert_eq!(winner.content, "new");
        assert_eq!(winner.modified_at, T3);
        assert_eq!(losers, vec![(String::new(), "old".into())]);
        assert!(resolve_versions(vec![]).is_none());
    }

    #[test]
    fn new_notes_travel_both_ways() {
        let local = note("here", T1);
        let plan = plan_note(NoteSides {
            local: Some(&local),
            ..Default::default()
        });
        assert_eq!(plan.remote, RemoteAction::Write(local.clone()));
        assert_eq!(plan.base, BaseAction::Set(local.clone()));

        let plan = plan_note(NoteSides {
            remote: Some(&local),
            ..Default::default()
        });
        assert_eq!(plan.local, LocalAction::Upsert(local.clone()));
        assert_eq!(plan.remote, RemoteAction::None);
    }

    #[test]
    fn unchanged_notes_need_nothing() {
        let n = note("same", T1);
        let plan = plan_note(NoteSides {
            local: Some(&n),
            remote: Some(&n),
            base: Some(&n),
            ..Default::default()
        });
        assert_eq!(plan.local, LocalAction::None);
        assert_eq!(plan.remote, RemoteAction::None);
        assert_eq!(plan.base, BaseAction::Keep);
    }

    #[test]
    fn deletions_win_only_over_older_edits() {
        let n = note("x", T2);

        // Deleted elsewhere after our last edit → delete here.
        let plan = plan_note(NoteSides {
            local: Some(&n),
            remote_tombstone: Some(T3),
            base: Some(&n),
            ..Default::default()
        });
        assert_eq!(plan.local, LocalAction::Delete);
        assert_eq!(plan.base, BaseAction::Clear);

        // Edited here after it was deleted elsewhere → keep and upload again.
        let plan = plan_note(NoteSides {
            local: Some(&n),
            remote_tombstone: Some(T1),
            ..Default::default()
        });
        assert_eq!(plan.local, LocalAction::None);
        assert_eq!(plan.remote, RemoteAction::Write(n.clone()));

        // Deleted here: remove the file and leave a tombstone…
        let plan = plan_note(NoteSides {
            local_tombstone: Some(T3),
            remote: Some(&n),
            base: Some(&n),
            ..Default::default()
        });
        assert_eq!(
            plan.remote,
            RemoteAction::Delete {
                tombstone: Some(T3.into())
            }
        );
        assert!(plan.clear_local_tombstone);

        // …unless another device edited it after that.
        let plan = plan_note(NoteSides {
            local_tombstone: Some(T1),
            remote: Some(&n),
            ..Default::default()
        });
        assert_eq!(plan.local, LocalAction::Upsert(n.clone()));

        // Deleted here, never synced: just the tombstone.
        let plan = plan_note(NoteSides {
            local_tombstone: Some(T1),
            ..Default::default()
        });
        assert_eq!(plan.remote, RemoteAction::Tombstone(T1.into()));
    }

    #[test]
    fn tag_colors_merge_per_tag() {
        let map = |pairs: &[(&str, &str)]| -> BTreeMap<String, String> {
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        };
        let base = map(&[("work", "blue"), ("home", "green")]);
        let local = map(&[("work", "red"), ("home", "green")]);
        let remote = map(&[("work", "blue"), ("idea", "pink")]);
        assert_eq!(
            merge_map(&base, &local, &remote),
            map(&[("work", "red"), ("idea", "pink")])
        );
    }
}
