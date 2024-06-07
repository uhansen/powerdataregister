use serde::{Deserialize};

#[derive(Deserialize)]
pub struct Customer {
  pub id: i64,
  pub firstName: String,
  pub lastName: String,
  pub street : String,
  pub city : String,
  pub zip : String,
  pub country : String,
  pub accessKey : String,    
}
