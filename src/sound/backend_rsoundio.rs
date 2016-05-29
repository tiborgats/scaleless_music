use rb::{RB, RbConsumer, RbProducer, SpscRb};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::{error, fmt};
use sound::*;

// TODO create this backend
