use crate::sometsuki::Sometsuki;
use std::sync::{Mutex, mpsc::Sender};

#[derive(Default)]
pub struct AppStateInner {
  pub sometsuki: Option<Sometsuki>,
  pub sometsuki_tx: Option<Sender<()>>
}
pub type AppState = Mutex<AppStateInner>;
