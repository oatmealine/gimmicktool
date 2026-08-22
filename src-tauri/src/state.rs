use crate::sometsuki::SometsukiCommand;
use std::sync::mpsc::Sender;

pub struct AppState {
  pub sometsuki_tx: Sender<SometsukiCommand>,
}