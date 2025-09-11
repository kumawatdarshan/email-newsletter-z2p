use super::SubscriberEmail;
use super::SubscriberName;

// #[derive(Debug, Deserialize, Validate)]
// #[validate(length(min = 1, max = 256), custom(function = parse_name))]
// #[validate(email)]
pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}
