use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub message: String,
    pub notification_type: NotificationType,
    pub created_at: SystemTime,
    pub timeout_ms: u64,
}

impl Notification {
    pub fn new(
        id: u64,
        message: String,
        notification_type: NotificationType,
        timeout_ms: u64,
    ) -> Self {
        Self {
            id,
            message,
            notification_type,
            created_at: SystemTime::now(),
            timeout_ms,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Ok(elapsed) = self.created_at.elapsed() {
            elapsed.as_millis() as u64 >= self.timeout_ms
        } else {
            false
        }
    }
}

pub struct NotificationManager {
    next_id: u64,
    pub notifications: Vec<Notification>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            notifications: Vec::new(),
        }
    }

    pub fn add(
        &mut self,
        message: String,
        notification_type: NotificationType,
        timeout_ms: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let notification = Notification::new(id, message, notification_type, timeout_ms);
        self.notifications.push(notification);

        id
    }

    pub fn remove(&mut self, id: u64) {
        self.notifications.retain(|n| n.id != id);
    }

    pub fn remove_expired(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }

    pub fn get_active(&self) -> Vec<&Notification> {
        self.notifications
            .iter()
            .filter(|n| !n.is_expired())
            .collect()
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_notification_creation() {
        let notif = Notification::new(1, "Test".to_string(), NotificationType::Info, 5000);
        assert_eq!(notif.id, 1);
        assert_eq!(notif.message, "Test");
        assert_eq!(notif.notification_type, NotificationType::Info);
        assert!(!notif.is_expired());
    }

    #[test]
    fn test_notification_expiration() {
        let notif = Notification::new(1, "Test".to_string(), NotificationType::Info, 100);
        assert!(!notif.is_expired());
        thread::sleep(Duration::from_millis(150));
        assert!(notif.is_expired());
    }

    #[test]
    fn test_manager_add_remove() {
        let mut manager = NotificationManager::new();
        let id1 = manager.add("Test 1".to_string(), NotificationType::Info, 5000);
        let id2 = manager.add("Test 2".to_string(), NotificationType::Warning, 5000);

        assert_eq!(manager.notifications.len(), 2);
        manager.remove(id1);
        assert_eq!(manager.notifications.len(), 1);
        assert_eq!(manager.notifications[0].id, id2);
    }

    #[test]
    fn test_manager_remove_expired() {
        let mut manager = NotificationManager::new();
        manager.add("Test 1".to_string(), NotificationType::Info, 100);
        manager.add("Test 2".to_string(), NotificationType::Warning, 5000);

        thread::sleep(Duration::from_millis(150));
        manager.remove_expired();

        assert_eq!(manager.notifications.len(), 1);
        assert_eq!(manager.notifications[0].message, "Test 2");
    }
}
