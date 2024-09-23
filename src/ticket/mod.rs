use futures::TryStreamExt;
use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Ticket {
    // Define the fields of the Ticket struct
    id: String,
    title: String,
    description: String,
}

pub struct TicketFunctions {
    collection: Collection<Ticket>,
}

impl TicketFunctions {
    pub fn new(client: Client) -> Self {
        let db = client.database("tickets");
        let collection = db.collection::<Ticket>("ticket");
        TicketFunctions { collection }
    }

    pub async fn create_ticket(&self, ticket: Ticket) -> Result<(), mongodb::error::Error> {
        self.collection.insert_one(ticket).await?;
        Ok(())
    }

    pub async fn get_all_tickets(&self) -> Result<Vec<Ticket>, mongodb::error::Error> {
        let mut tickets = Vec::new();
        let mut cursor = self.collection.find(doc! {}).await?;
        while let Some(ticket) = cursor.try_next().await? {
            tickets.push(ticket);
        }
        Ok(tickets)
    }
}
