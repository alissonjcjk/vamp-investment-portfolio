use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Asset {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub quantity: f64,
    pub unit_value: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Asset {
    pub fn total_value(&self) -> f64 {
        self.quantity * self.unit_value
    }
}

pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_total_value() {
        let asset = Asset {
            id: 1,
            user_id: 1,
            name: "BTC".to_string(),
            quantity: 2.0,
            unit_value: 3.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(asset.total_value(), 6.0);
    }
}
