use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: String, // "light", "dark", "system"
    pub font_size: i32,
    pub auto_save_interval: i32, // seconds
    pub git_auto_commit: bool,
    pub collaboration_enabled: bool,
    pub shortcuts: HashMap<String, String>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();
        // Default keyboard shortcuts
        shortcuts.insert("create_block".to_string(), "Enter".to_string());
        shortcuts.insert("indent_block".to_string(), "Tab".to_string());
        shortcuts.insert("outdent_block".to_string(), "Shift+Tab".to_string());
        shortcuts.insert("delete_block".to_string(), "Ctrl+D".to_string());
        shortcuts.insert("save".to_string(), "Ctrl+S".to_string());
        shortcuts.insert("undo".to_string(), "Ctrl+Z".to_string());
        shortcuts.insert("redo".to_string(), "Ctrl+Y".to_string());

        UserPreferences {
            theme: "system".to_string(),
            font_size: 14,
            auto_save_interval: 30, // 30 seconds
            git_auto_commit: true,
            collaboration_enabled: false, // Phase 1: disabled by default
            shortcuts,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User identifier
    pub id: String,
    
    /// Display information
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    
    /// User preferences
    pub preferences: UserPreferences,
    
    /// Collaboration status (for future phases)
    pub is_online: bool,
    pub last_seen: DateTime<Utc>,
    
    /// Authentication (for future phases)
    pub auth_token: Option<String>,
    
    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user with default preferences
    pub fn new(name: String, email: Option<String>) -> Self {
        let now = Utc::now();
        let id = crate::utils::generate_id();

        User {
            id,
            name,
            email,
            avatar_url: None,
            preferences: UserPreferences::default(),
            is_online: false, // Phase 1: always offline
            last_seen: now,
            auth_token: None, // Phase 1: no authentication
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a local user (for single-user Phase 1)
    pub fn create_local_user() -> Self {
        Self::new("Local User".to_string(), None)
    }

    /// Update user preferences
    pub fn update_preferences(&mut self, preferences: UserPreferences) {
        self.preferences = preferences;
        self.updated_at = Utc::now();
    }

    /// Update user profile
    pub fn update_profile(&mut self, name: Option<String>, email: Option<String>, avatar_url: Option<String>) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(email) = email {
            self.email = Some(email);
        }
        if let Some(avatar_url) = avatar_url {
            self.avatar_url = Some(avatar_url);
        }
        self.updated_at = Utc::now();
    }

    /// Set online status (for future collaboration phases)
    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
        if online {
            self.last_seen = Utc::now();
        }
    }

    /// Update last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
    }

    /// Set a preference shortcut
    pub fn set_shortcut(&mut self, action: String, shortcut: String) {
        self.preferences.shortcuts.insert(action, shortcut);
        self.updated_at = Utc::now();
    }

    /// Get a preference shortcut
    pub fn get_shortcut(&self, action: &str) -> Option<&String> {
        self.preferences.shortcuts.get(action)
    }

    /// Update theme preference
    pub fn set_theme(&mut self, theme: String) {
        self.preferences.theme = theme;
        self.updated_at = Utc::now();
    }

    /// Update font size preference
    pub fn set_font_size(&mut self, font_size: i32) {
        self.preferences.font_size = font_size.clamp(8, 72); // Reasonable bounds
        self.updated_at = Utc::now();
    }

    /// Update auto-save interval
    pub fn set_auto_save_interval(&mut self, interval: i32) {
        self.preferences.auto_save_interval = interval.max(5); // Minimum 5 seconds
        self.updated_at = Utc::now();
    }
}

/// Request to create a new user
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: Option<String>,
    pub preferences: Option<UserPreferences>,
}

/// Request to update user profile
#[derive(Debug, Deserialize)]
pub struct UpdateUserProfileRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// Request to update user preferences
#[derive(Debug, Deserialize)]
pub struct UpdateUserPreferencesRequest {
    pub theme: Option<String>,
    pub font_size: Option<i32>,
    pub auto_save_interval: Option<i32>,
    pub git_auto_commit: Option<bool>,
    pub collaboration_enabled: Option<bool>,
    pub shortcuts: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("Test User".to_string(), Some("test@example.com".to_string()));
        
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert_eq!(user.preferences.theme, "system");
        assert_eq!(user.preferences.font_size, 14);
        assert!(!user.is_online);
        assert!(user.auth_token.is_none());
    }

    #[test]
    fn test_local_user_creation() {
        let user = User::create_local_user();
        
        assert_eq!(user.name, "Local User");
        assert_eq!(user.email, None);
        assert!(!user.is_online);
    }

    #[test]
    fn test_user_preferences_update() {
        let mut user = User::create_local_user();
        
        let mut new_prefs = UserPreferences::default();
        new_prefs.theme = "dark".to_string();
        new_prefs.font_size = 16;
        
        let original_updated = user.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        user.update_preferences(new_prefs);
        
        assert_eq!(user.preferences.theme, "dark");
        assert_eq!(user.preferences.font_size, 16);
        assert!(user.updated_at > original_updated);
    }

    #[test]
    fn test_user_profile_update() {
        let mut user = User::create_local_user();
        
        user.update_profile(
            Some("Updated User".to_string()),
            Some("updated@example.com".to_string()),
            Some("https://example.com/avatar.png".to_string())
        );
        
        assert_eq!(user.name, "Updated User");
        assert_eq!(user.email, Some("updated@example.com".to_string()));
        assert_eq!(user.avatar_url, Some("https://example.com/avatar.png".to_string()));
    }

    #[test]
    fn test_user_shortcuts() {
        let mut user = User::create_local_user();
        
        user.set_shortcut("test_action".to_string(), "Ctrl+T".to_string());
        
        assert_eq!(user.get_shortcut("test_action"), Some(&"Ctrl+T".to_string()));
        assert_eq!(user.get_shortcut("nonexistent"), None);
    }

    #[test]
    fn test_user_theme_update() {
        let mut user = User::create_local_user();
        
        user.set_theme("dark".to_string());
        assert_eq!(user.preferences.theme, "dark");
    }

    #[test]
    fn test_user_font_size_bounds() {
        let mut user = User::create_local_user();
        
        user.set_font_size(100); // Should be clamped to 72
        assert_eq!(user.preferences.font_size, 72);
        
        user.set_font_size(5); // Should be clamped to 8
        assert_eq!(user.preferences.font_size, 8);
        
        user.set_font_size(16); // Should be accepted as-is
        assert_eq!(user.preferences.font_size, 16);
    }

    #[test]
    fn test_auto_save_interval_bounds() {
        let mut user = User::create_local_user();
        
        user.set_auto_save_interval(1); // Should be increased to minimum 5
        assert_eq!(user.preferences.auto_save_interval, 5);
        
        user.set_auto_save_interval(60); // Should be accepted as-is
        assert_eq!(user.preferences.auto_save_interval, 60);
    }

    #[test]
    fn test_default_shortcuts() {
        let prefs = UserPreferences::default();
        
        assert!(prefs.shortcuts.contains_key("create_block"));
        assert!(prefs.shortcuts.contains_key("indent_block"));
        assert!(prefs.shortcuts.contains_key("save"));
        assert!(prefs.shortcuts.contains_key("undo"));
        assert!(prefs.shortcuts.contains_key("redo"));
    }
} 