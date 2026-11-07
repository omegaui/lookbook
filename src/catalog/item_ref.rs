use super::ItemKind;

/// Points to one catalog item. It travels through actions as text like "distro:fedora".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRef {
    pub kind: ItemKind,
    pub id: String,
}

impl ItemRef {
    pub fn new(kind: ItemKind, id: &str) -> Self {
        Self {
            kind,
            id: id.to_owned(),
        }
    }

    pub fn to_action_target(&self) -> String {
        format!("{}:{}", self.kind.id(), self.id)
    }

    pub fn parse(target: &str) -> Option<Self> {
        let (kind, id) = target.split_once(':')?;
        let kind = ItemKind::from_id(kind)?;
        Some(Self::new(kind, id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_action_target() {
        let item = ItemRef::new(ItemKind::Compositor, "cosmic-comp");
        let target = item.to_action_target();

        assert_eq!(target, "compositor:cosmic-comp");
        assert_eq!(ItemRef::parse(&target), Some(item));
    }

    #[test]
    fn rejects_unknown_kinds() {
        assert_eq!(ItemRef::parse("planet:earth"), None);
        assert_eq!(ItemRef::parse("fedora"), None);
    }
}
