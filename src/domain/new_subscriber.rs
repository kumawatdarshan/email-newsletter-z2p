use super::SubscriberEmail;
use super::SubscriberName;

// #[derive(Debug, Deserialize, Validate)]
pub struct NewSubscriber {
    // #[validate(length(min = 1, max = 256), custom(function = parse_name))]
    pub name: SubscriberName,
    // #[validate(email)]
    pub email: SubscriberEmail,
}
