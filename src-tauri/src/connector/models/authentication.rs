use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AuthenticationKind {
   NONE,
   BASIC
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthenticationOptions {
   pub kind: AuthenticationKind,
   pub username: Option<String>,
   pub password: Option<String>,
   pub database: Option<String>
}


impl Default for AuthenticationOptions {
   fn default() -> Self {
       AuthenticationOptions { 
         kind: AuthenticationKind::NONE,
         username: None,
         password: None,
         database: None
      }
   }
}