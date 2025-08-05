# Corrected Feature Development Roadmap

## 🔧 **Key Changes: Unified Node Model**

This roadmap has been completely refactored to align with the new **unified node model**.

1.  **Journal-First Approach**: The app now opens directly to the daily journal.
2.  **Unified Node System**: Pages and blocks are replaced by a single `Node` entity.
3.  **Bi-Directional Linking**: A core feature from the beginning.
4.  **Simplified Phases**: The roadmap is now structured around building the core node experience first.

---

## **Phase 1: Foundation - The Journal & Node System (Weeks 1-8)**

### **1.1 Core Infrastructure & Unified Node Model (Weeks 1-3)**
*   **Planning & Design**:
    *   Finalize `Node` data structure and database schema (`nodes`, `node_links`).
    *   Design the Journal view and the "zoomed-in" Node view.
    *   Set dark theme as default.
*   **Implementation**:
    *   Set up Tauri project with a unified `Node` model in Rust.
    *   Implement backend `NodeService` with basic CRUD operations.
    *   Create a single `nodeStore` in Pinia for all state management.
    *   Build a recursive `Node` component in Vue to display a node and its children.

### **1.2 The Daily Journal (Weeks 4-5)**
*   **Planning & Design**:
    *   Design the daily journal UI, showing the node for the current date.
    *   Plan the logic for creating and retrieving daily notes.
*   **Implementation**:
    *   Implement `get_or_create_daily_note` Tauri command.
    *   Create the `JournalView.vue` that fetches and displays the daily note.
    *   Users can add, edit, and indent nodes directly in the journal.

### **1.3 Node Navigation & Zooming (Weeks 6-7)**
*   **Planning & Design**:
    *   Design the "zooming" behavior: clicking a node opens it as a new "page".
    *   Plan the routing system for ` /node/:nodeId`.
*   **Implementation**:
    *   Implement routing to `/node/:nodeId`.
    *   Create `NodeView.vue` that displays a specific node as the root.
    *   Clicking on any `[[wikilink]]` or node bullet navigates to that `NodeView`.

### **1.4 Basic Linking (Week 8)**
*   **Planning & Design**:
    *   Plan the backend logic for parsing `[[wikilinks]]` from node content.
*   **Implementation**:
    *   Implement backend `LinkService` to update `node_links` table on node creation/update.
    *   Make `[[wikilinks]]` in the UI clickable, navigating to the corresponding node.

---

## **Phase 2: Bi-Directional Linking & References (Weeks 9-12)**

### **2.1 Linked References (Weeks 9-10)**
*   **Planning & Design**:
    *   Design the "Linked References" section at the bottom of a `NodeView`.
*   **Implementation**:
    *   Implement `get_linked_references` Tauri command.
    *   Create a `LinkedReferences.vue` component to display the results.

### **2.2 Unlinked References (Weeks 11-12)**
*   **Planning & Design**:
    *   Design the "Unlinked References" section.
*   **Implementation**:
    *   Implement `get_unlinked_references` command using FTS5 search.
    *   Create an `UnlinkedReferences.vue` component.

---

## **Phase 3: The Library & Advanced Features (Weeks 13-18)**

### **3.1 The Library & Sidebar (Weeks 13-14)**
*   **Planning & Design**:
    *   Design the main sidebar for navigating between Journal and Library.
    *   The "Library" will be a view showing all nodes that are not daily notes.
*   **Implementation**:
    *   Build the main `Sidebar.vue`.
    *   Implement backend logic to fetch all non-journal nodes for the Library.

### **3.2 Full-Text Search (Weeks 15-16)**
*   **Planning & Design**:
    *   Design a global search command/UI.
*   **Implementation**:
    *   Implement FTS5 search commands in the backend.
    *   Create a `Search.vue` component.

### **3.3 Node Properties & Tags (Weeks 17-18)**
*   **Planning & Design**:
    *   Design the UI for adding and displaying properties on nodes (e.g., `key:: value`).
*   **Implementation**:
    *   Implement backend logic to parse and store properties.
    *   Update the `Node` component to display properties.

---

## **Phase 4: Git Integration & Polish (Weeks 19-24)**

### **4.1 Git Integration (Weeks 19-22)**
*   **Planning & Design**:
    *   Plan automatic Git commits for every change.
*   **Implementation**:
    *   Integrate `libgit2` into the backend.
    *   Set up automatic commits on every node create/update/delete.

### **4.2 UI/UX Polish (Weeks 23-24)**
*   **Planning & Design**:
    *   Review and refine all user interactions.
*   **Implementation**:
    *   Improve animations, transitions, and overall responsiveness.
    *   Finalize the dark theme.

---

## **Phase 5: Collaboration & Release (Weeks 25-30)**

(This phase will be planned in more detail later, but will involve CRDTs for real-time sync) 