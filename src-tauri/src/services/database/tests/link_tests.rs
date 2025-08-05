#[cfg(test)]
pub mod tests {
    use crate::services::{DatabaseService, LinkService};
    use crate::models::CreateNodeRequest;
    use tempfile::TempDir;

    fn setup() -> (TempDir, DatabaseService, LinkService) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();
        let db = DatabaseService::new_test(&db_path);
        db.init_database().unwrap();
        let link_service = LinkService::new(db.clone());
        (temp_dir, db, link_service)
    }

    #[test]
    fn test_update_links_for_node() {
        let (_temp_dir, db, link_service) = setup();

        // Create target nodes
        let target1 = db.create_node(CreateNodeRequest {
            content: "Target Node 1".to_string(),
            parent_id: None,
            order: None,
            properties: None,
            tags: None,
        }).unwrap();

        let target2 = db.create_node(CreateNodeRequest {
            content: "Target Node 2".to_string(),
            parent_id: None,
            order: None,
            properties: None,
            tags: None,
        }).unwrap();

        // Create source node with links
        let source = db.create_node(CreateNodeRequest {
            content: format!("This links to [[Target Node 1]] and [[Target Node 2]]"),
            parent_id: None,
            order: None,
            properties: None,
            tags: None,
        }).unwrap();

        // Update links
        link_service.update_links_for_node(&source).unwrap();

        // Get linked references for target1
        let refs1 = link_service.get_linked_references(&target1.id).unwrap();
        assert_eq!(refs1.len(), 1);
        assert_eq!(refs1[0].id, source.id);

        // Get linked references for target2
        let refs2 = link_service.get_linked_references(&target2.id).unwrap();
        assert_eq!(refs2.len(), 1);
        assert_eq!(refs2[0].id, source.id);
    }

    #[test]
    fn test_update_links_removes_old_links() {
        let (_temp_dir, db, link_service) = setup();

        // Create target nodes
        let target1 = db.create_node(CreateNodeRequest {
            content: "Target Node 1".to_string(),
            parent_id: None,
            order: None,
            properties: None,
            tags: None,
        }).unwrap();

        let target2 = db.create_node(CreateNodeRequest {
            content: "Target Node 2".to_string(),
            parent_id: None,
            order: None,
            properties: None,
            tags: None,
        }).unwrap();

        // Create source node with link to target1
        let source = db.create_node(CreateNodeRequest {
            content: format!("This links to [[Target Node 1]]"),
            parent_id: None,
            order: None,
            properties: None,
            tags: None,
        }).unwrap();

        // Update links
        link_service.update_links_for_node(&source).unwrap();

        // Verify link exists
        let refs1 = link_service.get_linked_references(&target1.id).unwrap();
        assert_eq!(refs1.len(), 1);

        // Update source to link to target2 instead
        let updated_source = db.update_node(&source.id, crate::models::UpdateNodeRequest {
            content: Some(format!("This links to [[Target Node 2]]")),
            properties: None,
            tags: None,
        }).unwrap();

        // Update links again
        link_service.update_links_for_node(&updated_source).unwrap();

        // Verify old link is removed
        let refs1_after = link_service.get_linked_references(&target1.id).unwrap();
        assert_eq!(refs1_after.len(), 0);

        // Verify new link exists
        let refs2_after = link_service.get_linked_references(&target2.id).unwrap();
        assert_eq!(refs2_after.len(), 1);
        assert_eq!(refs2_after[0].id, source.id);
    }

    #[test]
    fn test_get_unlinked_references() {
        let (_temp_dir, db, link_service) = setup();

        // Create nodes
        let node1 = db.create_node(CreateNodeRequest {
            content: "Test Node".to_string(),
            parent_id: None,
            order: None,
            properties: None,
            tags: None,
        }).unwrap();

        let node2 = db.create_node(CreateNodeRequest {
            content: "This mentions Test Node but doesn't link to it".to_string(),
            parent_id: None,
            order: None,
            properties: None,
            tags: None,
        }).unwrap();

        let node3 = db.create_node(CreateNodeRequest {
            content: "This also has Test Node in the content".to_string(),
            parent_id: None,
            order: None,
            properties: None,
            tags: None,
        }).unwrap();

        // Get unlinked references
        let unlinked = link_service.get_unlinked_references(&node1.id, "Test Node").unwrap();
        
        // Should find both nodes that mention "Test Node"
        assert_eq!(unlinked.len(), 3); // Including the original node itself
        let unlinked_ids: Vec<String> = unlinked.iter().map(|n| n.id.clone()).collect();
        assert!(unlinked_ids.contains(&node1.id));
        assert!(unlinked_ids.contains(&node2.id));
        assert!(unlinked_ids.contains(&node3.id));
    }

    #[test]
    fn test_links_with_nonexistent_targets() {
        let (_temp_dir, db, link_service) = setup();

        // Create source node with links to non-existent nodes
        let source = db.create_node(CreateNodeRequest {
            content: "This links to [[Non-existent Node]] and [[Another Missing Node]]".to_string(),
            parent_id: None,
            order: None,
            properties: None,
            tags: None,
        }).unwrap();

        // Update links - should not fail even if targets don't exist
        link_service.update_links_for_node(&source).unwrap();

        // Create one of the target nodes
        let target = db.create_node(CreateNodeRequest {
            content: "Non-existent Node".to_string(),
            parent_id: None,
            order: None,
            properties: None,
            tags: None,
        }).unwrap();

        // Update links again
        link_service.update_links_for_node(&source).unwrap();

        // Now the link should be established
        let refs = link_service.get_linked_references(&target.id).unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].id, source.id);
    }
}