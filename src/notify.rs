use crate::models::game::GameId;

pub enum Notification {
    ReleaseDateUpdated { game: GameId, prev: String, updated: String },
    // FIXME: Use appid for consistency; provide a way of looking up name here
    Released { game: String },
}

// Handles collecting and notifying about analytics / events
pub trait NotificationHandling {
    fn enqueue(&mut self, n: Notification) -> ();
    fn run(&self) -> ();
}

pub struct PrintNotifier {
    notifications: Vec<Notification>
}

impl PrintNotifier {
    pub fn new() -> PrintNotifier {
        PrintNotifier { notifications: vec![] }
    }
}

impl NotificationHandling for PrintNotifier {
    fn enqueue(&mut self, n: Notification) -> () {
        self.notifications.push(n);
    }

    fn run(&self) -> () {
        for n in &self.notifications {
            // N.B. I don't use Display because this needs to be updated to first resolve app_id
            // by running a query, anyway; Display won't have sufficient context.
            match n {
                Notification::ReleaseDateUpdated { game, prev, updated } =>
                    println!("🔎 Release date changed for {}: \"{}\" -> \"{}\"", &game, &prev, &updated),
                Notification::Released { game } =>
                    println!("🚀 {} is newly released!", &game)
            }
        }
    }
}
