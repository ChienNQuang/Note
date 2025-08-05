# Frontend Component Implementation (Refactored)

## 1. Component Architecture (Node-Centric)

### 1.1 Component Hierarchy
```
App.vue
├── Layout/
│   ├── AppSidebar.vue
│   └── MainContent.vue
├── Views/
│   ├── JournalView.vue
│   └── NodeView.vue
├── Node/
│   ├── Node.vue
│   ├── NodeContent.vue
│   ├── NodeProperties.vue
│   └── NodeToolbar.vue
├── References/
│   ├── LinkedReferences.vue
│   └── UnlinkedReferences.vue
└── UI/ (Shadcn-Vue components)
    ├── Button.vue
    ├── Input.vue
    └── ...
```

## 2. Core Components Implementation

### 2.1 App.vue - Main Container
- **Responsibility**: Holds the main layout, sidebar, and router-view.
- **Implementation**:
    - A simple layout with a `AppSidebar` component and a `router-view` to display either the `JournalView` or `NodeView`.

### 2.2 JournalView.vue - The Main Interface
- **Responsibility**: Displays the daily note for the current date.
- **Implementation**:
    - On mount, calls `get_or_create_daily_note` to fetch the main node for today.
    - Renders a single `Node` component, passing the daily note as a prop.
    - Handles date navigation (previous/next day).

### 2.3 NodeView.vue - The "Zoomed-In" View
- **Responsibility**: Displays a single node as the root of the view.
- **Implementation**:
    - Takes a `nodeId` as a prop from the router (`/node/:nodeId`).
    - Fetches the specified node and its children.
    - Renders the node's content as a title.
    - Renders the node's children using a list of `Node` components.
    - Includes the `LinkedReferences` and `UnlinkedReferences` components at the bottom.

### 2.4 Node.vue - The Recursive Node Component
- **Responsibility**: The core component for displaying and editing a single node and its children.
- **Implementation**:
    - Takes a `node` object as a prop.
    - Renders the node's content, making it editable.
    - Renders a list of child `Node` components for each child in `node.children`.
    - Handles all user interactions: editing content, creating new nodes (Enter), indenting (Tab), etc.
    - Emits events (`node-updated`, `node-created`, `node-deleted`) up to the parent view.

## 3. Pinia Stores Implementation (Refactored)

### 3.1 `nodeStore` - The Single Source of Truth
- **Responsibility**: Manages all node-related state in the application.
- **State**:
    - `nodes`: A flat map of `nodeId` to `node` object.
    - `openNodes`: A map of `nodeId` to an array of its children IDs, for nodes that are currently displayed.
- **Actions**:
    - `getNode(nodeId)`: Fetches a node from the backend and adds it to the `nodes` map.
    - `getChildren(nodeId)`: Fetches the children of a node and updates `openNodes`.
    - `createNode(data)`: Calls the backend to create a new node, then updates the state.
    - `updateNode(nodeId, data)`: Calls the backend to update a node.
    - ...other CRUD actions.

## 4. Services Layer Implementation

(Services like Markdown parsing will be simplified and integrated directly into the `nodeStore` or `Node` components as needed). 