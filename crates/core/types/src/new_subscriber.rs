use crate::subscriber_email::SubscriberEmail;
use crate::subscriber_name::SubscriberName;

#[derive(Debug)]
pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}
