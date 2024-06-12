use anyhow::{Context, Result};
use build_html::{Container, ContainerType, Html, HtmlContainer};
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Params, Request, Response, Router};
use spin_sdk::http_component;
use spin_sdk::sqlite::{Connection, Value};

/// A simple Spin HTTP component.
#[http_component]
fn handle_powerstatusapi(req: Request) -> anyhow::Result<impl IntoResponse> {
    let mut r = Router::default();
  r.post("/powerstatusapi/customers", add_new);
  r.get("/powerstatusapi/customers", get_all);
  r.delete("/powerstatusapi/customers/:id", delete_one);
  Ok(r.handle(req))
}

#[derive(Debug, Deserialize, Serialize)]
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

fn add_new(req: Request, _params: Params) -> anyhow::Result<impl IntoResponse> {
  let Ok(item): Result<Customer> =
      serde_json::from_reader(req.body()).with_context(|| "Error while deserializing payload")
    else {
      return Ok(Response::new(400, "Invalid payload received"));
    };
    let connection = Connection::open_default()?;
    let parameters = &[Value::Text(item.firstName), 
                    Value::Text(item.lastName), 
                    Value::Text(item.street), 
                    Value::Text(item.city), 
                    Value::Text(item.zip), 
                    Value::Text(item.country), 
                    Value::Text(item.accessKey)];
    connection.execute("INSERT INTO CUSTOMERS (FirstName,LastName, Street, City, Zip, Country, AccessKey) VALUES (?,?,?,?,?,?,?)", parameters)?;
    Ok(Response::builder()
        .status(200)
        .header("HX-Trigger", "newItem")
        .body(())
        .build())
  }

  
fn get_all(_r: Request, _p: Params) -> anyhow::Result<impl IntoResponse> {
    let connection = Connection::open_default()?;
    let row_set = connection.execute("SELECT ID, FirstName,LastName, Street, City, Zip, Country, AccessKey FROM CUSTOMERS ORDER BY ID DESC", &[])?;
  
    let items = row_set
      .rows()
      .map(|row| Customer {
        id: row.get::<i64>("ID").unwrap(),
        value: row.get::<&str>("VALUE").unwrap().to_owned(),
      })
      .map(|item| item.to_html_string())
      .reduce(|acc, e| format!("{} {}", acc, e))
      .unwrap_or(String::from(""));
  
    Ok(Response::builder()
      .status(200)
      .header("Content-Type", "text/html")
      .body(items)?)
  }

  fn delete_one(_req: Request, params: Params) -> anyhow::Result<impl IntoResponse> {
    let Some(id) = params.get("id") else {
      return Ok(Response::builder().status(404).body("Missing identifier")?);
    };
    let Ok(id) = id.parse::<i64>() else {
      return Ok(Response::builder()
        .status(400)
        .body("Unexpected identifier format"));
    };
    let connection = Connection::open_default()?;
    let parameters = &[Value::Integer(id)];
    
    match connection.execute("DELETE FROM CUSTOMERS WHERE ID = ?", parameters) {
        // HTMX requires status 200 instead of 204
        Ok(_) => Response::new(200, ()),
        Err(e) => {
            println!("Error while deleting item: {}", e);
            Response::builder()
                .status(500)
                .body("Error while deleting item")
                .build()
        }
    };}

impl Html for Customer {
  fn to_html_string(&self) -> String {
    Container::new(ContainerType::Div)
      .with_attributes(vec![
        ("class", "item"),
        ("id", format!("item-{}", &self.id).as_str()),
      ])
      .with_container(
        Container::new(ContainerType::Div)
          .with_attributes(vec![("class", "FirstName")])
          .with_raw(&self.firstName),
      )
      .with_container(
        Container::new(ContainerType::Div)
          .with_attributes(vec![
            ("class", "delete-item"),
            ("hx-delete", format!("/powerstatusapi/customers/{}", &self.id).as_str()),
          ])
          .with_raw("❌"),
      )
      .to_html_string()
  }}
