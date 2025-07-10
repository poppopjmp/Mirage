use crate::models::NotificationDelivery;
use mirage_common::Result;

// This channel is used for storing notifications in the database for later retrieval
// via API or dashboard. Since notifications are already stored in the DB by default,
// this implementation is a no-op.
pub struct DatabaseChannel;

impl DatabaseChannel {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::Channel for DatabaseChannel {
    async fn send(
        &self,
        _delivery: &NotificationDelivery,
        _content: &str,
        _subject: &str,
    ) -> Result<()> {
        // The notification is already stored in the database by the notification service
        // so this channel doesn't need to do anything else
        Ok(())
    }
}
