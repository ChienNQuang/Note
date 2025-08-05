use crate::models::{CreateNodeRequest, UpdateNodeRequest};
use crate::services::database::connection::DatabaseService;
use tempfile::tempdir;
use std::collections::HashMap;

#[test]
fn test_create_node() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    let request = CreateNodeRequest {
        content: "Test node content".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };

    let node = db.create_node(request).unwrap();
    
    assert!(!node.id.is_empty());
    assert_eq!(node.content, "Test node content");
    assert_eq!(node.parent_id, None);
    assert_eq!(node.order, 0);
    assert_eq!(node.created_by, "default_user");
    assert_eq!(node.version, 1);
}

#[test]
fn test_create_node_with_parent() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    // Create parent node
    let parent_request = CreateNodeRequest {
        content: "Parent node".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };
    let parent = db.create_node(parent_request).unwrap();

    // Create child node
    let child_request = CreateNodeRequest {
        content: "Child node".to_string(),
        parent_id: Some(parent.id.clone()),
        order: Some(1),
        properties: None,
        tags: Some(vec!["test-tag".to_string()]),
    };
    let child = db.create_node(child_request).unwrap();

    assert_eq!(child.parent_id, Some(parent.id));
    assert_eq!(child.order, 1);
    assert_eq!(child.tags, vec!["test-tag"]);
}

#[test]
fn test_get_node() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    let request = CreateNodeRequest {
        content: "Get test node".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };
    let created_node = db.create_node(request).unwrap();

    let retrieved_node = db.get_node(&created_node.id).unwrap();
    assert_eq!(retrieved_node.id, created_node.id);
    assert_eq!(retrieved_node.content, "Get test node");
}

#[test]
fn test_get_node_not_found() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    let result = db.get_node("non-existent-id");
    assert!(result.is_err());
}

#[test]
fn test_get_node_with_children() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    // Create parent node
    let parent_request = CreateNodeRequest {
        content: "Parent with children".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };
    let parent = db.create_node(parent_request).unwrap();

    // Create child nodes
    for i in 0..3 {
        let child_request = CreateNodeRequest {
            content: format!("Child {}", i),
            parent_id: Some(parent.id.clone()),
            order: Some(i),
            properties: None,
            tags: None,
        };
        db.create_node(child_request).unwrap();
    }

    let node_with_children = db.get_node_with_children(&parent.id).unwrap();
    assert_eq!(node_with_children.node.content, "Parent with children");
    assert_eq!(node_with_children.child_nodes.len(), 3);
    
    // Verify children are in correct order
    for (i, child) in node_with_children.child_nodes.iter().enumerate() {
        assert_eq!(child.node.content, format!("Child {}", i));
        assert_eq!(child.node.order, i as i32);
    }
}

#[test]
fn test_update_node_content() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    let request = CreateNodeRequest {
        content: "Original content".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };
    let node = db.create_node(request).unwrap();

    let update_request = UpdateNodeRequest {
        content: Some("Updated content".to_string()),
        properties: None,
        tags: None,
    };
    let updated_node = db.update_node(&node.id, update_request).unwrap();

    assert_eq!(updated_node.content, "Updated content");
    assert_eq!(updated_node.version, 2);
    assert!(updated_node.updated_at > node.updated_at);
}

#[test]
fn test_update_node_properties() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    let request = CreateNodeRequest {
        content: "Node with properties".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };
    let node = db.create_node(request).unwrap();

    let mut properties = HashMap::new();
    properties.insert("key1".to_string(), serde_json::json!("value1"));
    properties.insert("key2".to_string(), serde_json::json!(123));

    let update_request = UpdateNodeRequest {
        content: None,
        properties: Some(properties.clone()),
        tags: None,
    };
    let updated_node = db.update_node(&node.id, update_request).unwrap();

    assert_eq!(updated_node.properties, properties);
    assert_eq!(updated_node.content, "Node with properties"); // Content unchanged
}

#[test]
fn test_delete_node() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    let request = CreateNodeRequest {
        content: "Node to delete".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };
    let node = db.create_node(request).unwrap();

    // Delete the node
    db.delete_node(&node.id).unwrap();

    // Verify node is deleted
    let result = db.get_node(&node.id);
    assert!(result.is_err());
}

#[test]
fn test_delete_node_cascades_children() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    // Create parent and child nodes
    let parent_request = CreateNodeRequest {
        content: "Parent to delete".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };
    let parent = db.create_node(parent_request).unwrap();

    let child_request = CreateNodeRequest {
        content: "Child to be cascaded".to_string(),
        parent_id: Some(parent.id.clone()),
        order: Some(0),
        properties: None,
        tags: None,
    };
    let child = db.create_node(child_request).unwrap();

    // Delete parent
    db.delete_node(&parent.id).unwrap();

    // Verify both are deleted
    assert!(db.get_node(&parent.id).is_err());
    assert!(db.get_node(&child.id).is_err());
}

#[test]
fn test_move_node() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    // Create nodes
    let parent1_request = CreateNodeRequest {
        content: "Parent 1".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };
    let parent1 = db.create_node(parent1_request).unwrap();

    let parent2_request = CreateNodeRequest {
        content: "Parent 2".to_string(),
        parent_id: None,
        order: Some(1),
        properties: None,
        tags: None,
    };
    let parent2 = db.create_node(parent2_request).unwrap();

    let child_request = CreateNodeRequest {
        content: "Child to move".to_string(),
        parent_id: Some(parent1.id.clone()),
        order: Some(0),
        properties: None,
        tags: None,
    };
    let child = db.create_node(child_request).unwrap();

    // Move child from parent1 to parent2
    db.move_node(&child.id, Some(parent2.id.clone()), 5).unwrap();

    let moved_child = db.get_node(&child.id).unwrap();
    assert_eq!(moved_child.parent_id, Some(parent2.id));
    assert_eq!(moved_child.order, 5);
}

#[test]
fn test_move_node_to_root() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    // Create parent and child
    let parent_request = CreateNodeRequest {
        content: "Parent".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };
    let parent = db.create_node(parent_request).unwrap();

    let child_request = CreateNodeRequest {
        content: "Child to move to root".to_string(),
        parent_id: Some(parent.id.clone()),
        order: Some(0),
        properties: None,
        tags: None,
    };
    let child = db.create_node(child_request).unwrap();

    // Move child to root
    db.move_node(&child.id, None, 10).unwrap();

    let moved_child = db.get_node(&child.id).unwrap();
    assert_eq!(moved_child.parent_id, None);
    assert_eq!(moved_child.order, 10);
}

#[test]
fn test_get_daily_note() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    let date = "2024-01-15";
    let journal_request = CreateNodeRequest {
        content: date.to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: Some(vec!["#Journal".to_string()]),
    };
    let journal = db.create_node(journal_request).unwrap();

    let retrieved = db.get_daily_note(date).unwrap();
    assert_eq!(retrieved.id, journal.id);
    assert_eq!(retrieved.content, date);
    assert!(retrieved.tags.contains(&"#Journal".to_string()));
}

#[test]
fn test_get_or_create_daily_note_existing() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    let date = "2024-01-16";
    let journal_request = CreateNodeRequest {
        content: date.to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: Some(vec!["#Journal".to_string()]),
    };
    let journal = db.create_node(journal_request).unwrap();

    let retrieved = db.get_or_create_daily_note(date).unwrap();
    assert_eq!(retrieved.id, journal.id);
}

#[test]
fn test_get_or_create_daily_note_new() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    let date = "2024-01-17";
    let journal = db.get_or_create_daily_note(date).unwrap();
    
    assert_eq!(journal.content, date);
    assert!(journal.tags.contains(&"#Journal".to_string()));
    assert_eq!(journal.parent_id, None);
}

#[test]
fn test_node_hierarchy_integrity() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    // Create a three-level hierarchy
    let root_request = CreateNodeRequest {
        content: "Root node".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };
    let root = db.create_node(root_request).unwrap();

    let level1_request = CreateNodeRequest {
        content: "Level 1 node".to_string(),
        parent_id: Some(root.id.clone()),
        order: Some(0),
        properties: None,
        tags: None,
    };
    let level1 = db.create_node(level1_request).unwrap();

    let level2_request = CreateNodeRequest {
        content: "Level 2 node".to_string(),
        parent_id: Some(level1.id.clone()),
        order: Some(0),
        properties: None,
        tags: None,
    };
    let level2 = db.create_node(level2_request).unwrap();

    // Get the complete hierarchy
    let hierarchy = db.get_node_with_children(&root.id).unwrap();
    
    assert_eq!(hierarchy.child_nodes.len(), 1);
    assert_eq!(hierarchy.child_nodes[0].node.id, level1.id);
    assert_eq!(hierarchy.child_nodes[0].child_nodes.len(), 1);
    assert_eq!(hierarchy.child_nodes[0].child_nodes[0].node.id, level2.id);
}

#[test]
fn test_concurrent_updates() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = DatabaseService::new_test(db_path.to_str().unwrap());
    db.init_database().unwrap();

    let request = CreateNodeRequest {
        content: "Concurrent test".to_string(),
        parent_id: None,
        order: Some(0),
        properties: None,
        tags: None,
    };
    let node = db.create_node(request).unwrap();

    // Simulate concurrent updates
    let update1 = UpdateNodeRequest {
        content: Some("Update 1".to_string()),
        properties: None,
        tags: None,
    };
    let updated1 = db.update_node(&node.id, update1).unwrap();
    assert_eq!(updated1.version, 2);

    let update2 = UpdateNodeRequest {
        content: Some("Update 2".to_string()),
        properties: None,
        tags: None,
    };
    let updated2 = db.update_node(&node.id, update2).unwrap();
    assert_eq!(updated2.version, 3);
    assert_eq!(updated2.content, "Update 2");
}