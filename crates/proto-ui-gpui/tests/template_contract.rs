use std::collections::{BTreeMap, BTreeSet};

use proto_ui_gpui::{BridgeError, SemanticId, TemplateNode, TemplateSnapshot, prune_replaced_tree};

#[test]
fn nested_template_round_trips_slot_and_svg_without_structural_ids() {
    let mut attributes = BTreeMap::new();
    attributes.insert("viewBox".to_owned(), "0 0 24 24".to_owned());
    attributes.insert("width".to_owned(), "16".to_owned());
    let nodes = vec![TemplateNode::Container {
        tag: "button".to_owned(),
        style: vec!["inline-flex".to_owned()],
        children: vec![
            TemplateNode::Text {
                text: "before".to_owned(),
            },
            TemplateNode::Slot {
                slot_id: "button-slot".to_owned(),
            },
            TemplateNode::Svg {
                tag: "svg".to_owned(),
                attributes,
                children: vec![TemplateNode::Svg {
                    tag: "path".to_owned(),
                    attributes: BTreeMap::from([(String::from("d"), String::from("M0 0"))]),
                    children: vec![],
                }],
            },
        ],
    }];
    let value = serde_json::to_value(&nodes).expect("template serializes");
    assert_eq!(value[0]["kind"], "container");
    assert_eq!(value[0]["children"][2]["kind"], "svg");
    let decoded: Vec<TemplateNode> = serde_json::from_value(value).expect("template decodes");
    assert_eq!(decoded, nodes);

    let snapshot = TemplateSnapshot::try_new(
        decoded,
        vec![SemanticId::new("button").expect("semantic id")],
    )
    .expect("snapshot validates");
    assert_eq!(snapshot.semantic_ids.len(), 1);
    assert!(matches!(snapshot.nodes[0], TemplateNode::Container { .. }));
}

#[test]
fn duplicate_semantic_ids_fail_and_removed_ids_are_pruned() {
    let retained = SemanticId::new("retained").expect("semantic id");
    let removed = SemanticId::new("removed").expect("semantic id");
    let previous = TemplateSnapshot::try_new(
        vec![TemplateNode::slot("slot")],
        vec![retained.clone(), removed.clone()],
    )
    .expect("previous snapshot");
    let next = TemplateSnapshot::try_new(vec![TemplateNode::slot("slot")], vec![retained.clone()])
        .expect("next snapshot");
    assert_eq!(prune_replaced_tree(&previous, &next), vec![removed.clone()]);

    assert!(matches!(
        TemplateSnapshot::try_new(vec![], vec![retained.clone(), retained],),
        Err(BridgeError::DuplicateSemanticId { .. })
    ));
    let expected: BTreeSet<_> = [removed].into_iter().collect();
    assert_eq!(pruned_set(&previous, &next), expected);
}

fn pruned_set(previous: &TemplateSnapshot, next: &TemplateSnapshot) -> BTreeSet<SemanticId> {
    prune_replaced_tree(previous, next).into_iter().collect()
}

#[test]
fn empty_semantic_ids_are_rejected() {
    assert!(matches!(
        SemanticId::new(" "),
        Err(BridgeError::InvalidIdentity { .. })
    ));
}
