use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug)]
pub struct Duration(std::time::Duration);
impl Duration {
    pub fn from_secs(secs: u64) -> Self {
        Self(std::time::Duration::from_secs(secs))
    }
}
impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.as_secs().to_string())
    }
}

#[derive(Debug, Serialize, Default)]
pub struct APNSFcmOptions {
    /// Label associated with the message's analytics data.
    #[serde(skip_serializing_if = "Option::is_none")]
    analytics_label: Option<String>,
    /// Contains the URL of an image that is going to be displayed in a notification.
    /// If present, it will override [[MessageLike]]::fcmOptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
}

#[derive(Debug, Serialize, Default)]
/// APNs HTTP headers properties
/// See https://developer.apple.com/documentation/usernotifications/sending-notification-requests-to-apns
pub struct ApnsHeaders {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<String>,
    #[serde(rename = "apns-id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// A canonical UUID that’s the unique ID for the notification.
    /// If an error occurs when sending the notification,
    /// APNs includes this value when reporting the error to your server.
    /// Canonical UUIDs are 32 lowercase hexadecimal digits, displayed in five groups
    /// separated by hyphens in the form 8-4-4-4-12.
    /// For example: 123e4567-e89b-12d3-a456-4266554400a0.
    ///
    /// If you omit this header, APNs creates a UUID for you and returns it in its response.
    pub apns_id: Option<String>,
    #[serde(rename = "apns-push-type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The value of this header must accurately reflect the contents of your notification’s payload.
    /// If there’s a mismatch, or if the header is missing on required systems, APNs may return an error,
    /// delay the delivery of the notification, or drop it altogether.
    pub apns_push_type: Option<ApnsPushType>,
    /// The date at which the notification is no longer valid. This value is
    /// a UNIX epoch expressed in seconds (UTC).
    ///
    /// If the value is nonzero, APNs stores the notification and tries to deliver it at least once,
    /// repeating the attempt as needed until the specified date.
    ///
    /// If the value is 0, APNs attempts to deliver the notification only once and doesn’t store it.
    ///
    /// If you omit this header, APNs stores the push according to APNs storage policy.
    #[serde(rename = "apns-expiration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apns_expiration: Option<Duration>,
    /// The priority of the notification.
    ///
    /// If you omit this header, APNs sets the notification priority to 10.
    ///
    /// - Specify 10 to send the notification immediately.
    /// - Specify 5 to send the notification based on power considerations on the user’s device.
    /// - Specify 1 to prioritize the device’s power considerations over all other
    /// factors for delivery, and prevent awakening the device.
    #[serde(rename = "apns-priority")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apns_priority: Option<ApnsPriority>,
    /// The topic for the notification. In general, the topic is your app’s bundle ID/app ID.
    /// It can have a suffix based on the type of push notification.
    ///
    /// If you’re using token-based authentication with APNs, you must include
    /// this header with the correct bundle ID and suffix combination.
    #[serde(rename = "apns-topic")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apns_topic: Option<String>,
    /// An identifier you use to merge multiple notifications into a single notification for the user.
    /// Typically, each notification request displays a new notification on the user’s device.
    ///
    /// When sending the same notification more than once, use the same value in this
    /// header to merge the requests.
    /// The value of this key must not exceed 64 bytes.
    #[serde(rename = "apns-collapse-id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apns_collapse_id: Option<String>,
}

impl ApnsHeaders {
    pub fn ios_background_notification() -> ApnsHeaders {
        ApnsHeaders {
            apns_push_type: Some(ApnsPushType::Background),
            apns_priority: Some(ApnsPriority::RespectEnergySavingMode),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize)]
pub enum ApnsPriority {
    #[serde(rename = "10")]
    SendImmediately,
    #[serde(rename = "5")]
    RespectEnergySavingMode,
    #[serde(rename = "1")]
    RespectEnergySavingModeNoAwaking,
}

#[derive(Debug, Serialize, Default)]
pub struct ApnsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    headers: Option<ApnsHeaders>,
}

impl ApnsConfig {
    pub fn new(
        aps: &Aps,
        data: &HashMap<String, String>,
        headers: Option<ApnsHeaders>,
    ) -> ApnsConfig {
        let mut payload = serde_json::json!({
            "aps": aps,
        });
        let data_payload = serde_json::json!(data);
        ApnsConfig::merge(&mut payload, &data_payload);
        ApnsConfig {
            payload: Some(payload),
            headers,
        }
    }
    pub fn ios_background_notification(data_payload: HashMap<String, String>) -> ApnsConfig {
        let mut payload = serde_json::json!({
            "aps": Aps {
                content_available: Some(ContentAvailable::On),
                ..Default::default()
            }
        });
        let data_payload = serde_json::json!(data_payload);
        ApnsConfig::merge(&mut payload, &data_payload);

        ApnsConfig {
            payload: Some(payload),
            headers: Some(ApnsHeaders::ios_background_notification()),
        }
    }
    fn merge(a: &mut serde_json::Value, b: &serde_json::Value) {
        match (a, b) {
            (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                for (k, v) in b {
                    ApnsConfig::merge(a.entry(k.clone()).or_insert(serde_json::Value::Null), v);
                }
            }
            (a, b) => *a = b.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ApnsPushType {
    /// The push type for notifications that trigger a user interaction—for example, an alert, badge, or sound.
    /// If you set this push type, the apns-topic header field must use your app’s bundle ID as the topic.
    /// For more information, refer to
    /// [Generating a remote notification](https://developer.apple.com/documentation/usernotifications/generating-a-remote-notification).
    /// If the notification requires immediate action from the user, set notification priority to 10; otherwise use 5.
    ///
    /// You’re required to use the alert push type on watchOS 6 and later. It’s recommended on macOS, iOS, tvOS, and iPadOS.
    Alert,
    /// The push type for notifications that deliver content in the background, and don’t trigger any user interactions.
    /// If you set this push type, the apns-topic header field must use your app’s bundle ID as the topic. Always use priority 5.
    /// Using priority 10 is an error. For more information, refer to
    /// [Pushing background updates to your App](https://developer.apple.com/documentation/usernotifications/pushing-background-updates-to-your-app).
    ///
    /// You’re required to use the background push type on watchOS 6 and later. It’s recommended on macOS, iOS, tvOS, and iPadOS.
    Background,
    /// The push type for notifications that request a user’s location. If you set this push type, the apns-topic
    /// header field must use your app’s bundle ID with.location-query appended to the end. For more information, refer to
    /// [Creating a location push service extension](https://developer.apple.com/documentation/CoreLocation/creating-a-location-push-service-extension).
    ///
    /// The location push type isn’t available on macOS, tvOS, and watchOS. It’s recommended for iOS and iPadOS.
    ///
    /// If the location query requires an immediate response from the Location Push Service Extension,
    /// set notification apns-priority to 10; otherwise, use 5. The location push type supports only token-based authentication.
    Location,
    /// The push type for notifications that provide information about an incoming Voice-over-IP (VoIP) call.
    /// For more information, refer to
    /// [Responding to VoIP Notifications from PushKit](https://developer.apple.com/documentation/PushKit/responding-to-voip-notifications-from-pushkit).
    /// If you set this push type, the apns-topic header field must use your app’s bundle ID with.voip appended to the end.
    ///
    /// If you’re using certificate-based authentication, you must also register the certificate for VoIP services.
    /// The topic is then part of the 1.2.840.113635.100.6.3.4 or 1.2.840.113635.100.6.3.6 extension.
    ///
    /// The voip push type isn’t available on watchOS. It’s recommended on macOS, iOS, tvOS, and iPadOS.
    VoiP,
    /// The push type for notifications that contain update information for a watchOS app’s complications.
    /// For more information, refer to
    /// [Keeping your complications up to date](https://developer.apple.com/documentation/clockkit/deprecated_articles_and_symbols/keeping_your_complications_up_to_date).
    ///
    /// If you set this push type, the apns-topic header field must use your app’s bundle ID with.complication
    /// appended to the end. If you’re using certificate-based authentication, you must also register
    /// the certificate for WatchKit services.
    ///
    /// The topic is then part of the 1.2.840.113635.100.6.3.6 extension.
    ///
    /// The complication push type isn’t available on macOS, tvOS, and iPadOS. It’s recommended for watchOS and iOS.
    Compilation,
    /// The push type to signal changes to a File Provider extension.
    ///
    /// If you set this push type, the apns-topic header field must use your app’s bundle ID with.pushkit.fileprovider
    /// appended to the end.
    ///
    /// For more information, refer to
    /// [Using push notifications to signal changes](https://developer.apple.com/documentation/FileProvider/using-push-notifications-to-signal-changes).
    ///
    /// The fileprovider push type isn’t available on watchOS. It’s recommended on macOS, iOS, tvOS, and iPadOS.
    FileProvider,
    /// The push type for notifications that tell managed devices to contact the MDM server.
    ///
    /// If you set this push type, you must use the topic from the UID attribute in the subject
    /// of your MDM push certificate.
    ///
    /// For more information, refer to
    /// [Device Management](https://developer.apple.com/documentation/devicemanagement).
    ///
    /// The mdm push type isn’t available on watchOS. It’s recommended on macOS, iOS, tvOS, and iPadOS.
    MDM,
    /// The push type to signal changes to a live activity session. If you set this push type,
    /// the apns-topic header field must use your app’s bundle ID with.push-type.liveactivity
    /// appended to the end. For more information, refer to Updating and ending your Live Activity
    /// with ActivityKit push notifications.
    ///
    /// The liveactivity push type isn’t available on watchOS, macOS, and tvOS. It’s recommended on iOS and iPadOS.
    LiveActivity,
    /// The push type for notifications that provide information about updates to your application’s
    /// push to talk services. For more information, refer to [Push to Talk](https://developer.apple.com/documentation/PushToTalk).
    ///
    /// If you set this push type, the apns-topic header field must use your app’s bundle ID with.voip-ptt appended to the end.
    ///
    /// The pushtotalk push type isn’t available on watchOS, macOS, and tvOS. It’s recommended on iOS and iPadOS.
    PushToTalk,
}

/// See https://developer.apple.com/documentation/usernotifications/generating-a-remote-notification
#[derive(Debug, Serialize, Default)]
pub struct Aps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<Alert>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<u32>,
    #[serde(rename = "thread-id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(rename = "content-available")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_available: Option<ContentAvailable>,
    #[serde(rename = "mutable-content")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutable_content: Option<MutableContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(rename = "dismissal-date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismissal_date: Option<u32>,
    #[serde(rename = "attributes-type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes_type: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum MutableContent {
    On,
    Off,
}

impl Serialize for MutableContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::On => 1.serialize(serializer),
            Self::Off => 0.serialize(serializer),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ContentAvailable {
    On,
    Off,
}

impl Serialize for ContentAvailable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ContentAvailable::On => 1.serialize(serializer),
            ContentAvailable::Off => 0.serialize(serializer),
        }
    }
}

#[derive(Debug)]
pub enum Alert {
    Simple(String),
    Structural(Box<RichAlert>),
}

impl Serialize for Alert {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Simple(alert) => alert.serialize(serializer),
            Self::Structural(alert) => alert.serialize(serializer),
        }
    }
}

#[derive(Debug, Serialize, Default)]
pub struct RichAlert {
    /// The title of the notification. Apple Watch displays this string in
    /// the short look notification interface. Specify a string that’s quickly
    /// understood by the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Additional information that explains the purpose of the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// The content of the alert message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// The name of the launch image file to display. If the user chooses
    /// to launch your app, the contents of the specified image or storyboard
    /// file are displayed instead of your app’s normal launch image.
    #[serde(rename = "launch-image")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_image: Option<String>,
    /// The key for a localized title string. Specify this key instead of the
    /// title key to retrieve the title from your app’s Localizable.strings files.
    /// The value must contain the name of a key in your strings file.
    #[serde(rename = "title-loc-key")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_loc_key: Option<String>,
    /// An array of strings containing replacement values for variables in your
    /// title string. Each %@ character in the string specified by the title-loc-key
    /// is replaced by a value from this array.
    /// The first item in the array replaces the first instance of the %@ character
    /// in the string, the second item replaces the second instance, and so on.
    #[serde(rename = "title-loc-args")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_loc_args: Option<Vec<String>>,
    /// The key for a localized subtitle string. Use this key, instead of the subtitle key,
    /// to retrieve the subtitle from your app’s Localizable.strings file.
    /// The value must contain the name of a key in your strings file.
    #[serde(rename = "subtitle-loc-key")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle_loc_key: Option<String>,
    /// An array of strings containing replacement values for variables in your title
    /// string. Each %@ character in the string specified by subtitle-loc-key is
    /// replaced by a value from this array. The first item in the array replaces
    /// the first instance of the %@ character in the string, the second item
    /// replaces the second instance, and so on.
    #[serde(rename = "subtitle-loc-args")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle_loc_args: Option<Vec<String>>,
    /// The key for a localized message string. Use this key, instead of
    /// the body key, to retrieve the message text from your app’s Localizable.strings
    /// file. The value must contain the name of a key in your strings file.
    #[serde(rename = "loc-key")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc_key: Option<String>,
    /// An array of strings containing replacement values for variables in your message text.
    /// Each %@ character in the string specified by loc-key is replaced by a value from this array.
    /// The first item in the array replaces the first instance of the %@ character in the string,
    /// the second item replaces the second instance, and so on.
    #[serde(rename = "loc-args")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc_args: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum Sound {
    Simple(String),
    Structural {
        critical: u8,
        /// The name of a sound file in your app’s main bundle or in the Library/Sounds
        /// folder of your app’s container directory. Specify the string “default” to play
        /// the system sound. For information about how to prepare sounds,
        /// see [UNNotificationSound](https://developer.apple.com/documentation/usernotifications/unnotificationsound).
        name: String,
        /// The volume for the critical alert’s sound.
        /// Set this to a value between 0 (silent) and 1 (full volume).
        volume: f32,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::fcm::ios::RichAlert;

    use super::{Alert, ApnsConfig};

    #[test]
    fn check_serialization_for_union_like_type() {
        let simple = Alert::Simple("bar".to_string());
        let json = serde_json::json!({
            "foo": simple
        });
        assert_eq!(
            json,
            serde_json::json!({
                "foo": "bar"
            })
        );
        let structural = Alert::Structural(
            RichAlert {
                title: Some("title".to_string()),
                subtitle: Some("subtitle".to_string()),
                body: Some("body".to_string()),
                ..Default::default()
            }
            .into(),
        );
        let json = serde_json::json!({
            "foo": structural
        });
        assert_eq!(
            json,
            serde_json::json!({
                "foo": {
                    "title": "title",
                    "subtitle": "subtitle",
                    "body": "body"
                }
            })
        )
    }
    #[test]
    fn check_serialization_for_ios_background_type() {
        let payload = ApnsConfig::ios_background_notification(HashMap::from_iter([(
            "example".to_string(),
            "example".to_string(),
        )]));
        let json = serde_json::json!(payload);
        let expect = serde_json::json!({
            "headers": {
                "apns-push-type": "background",
                "apns-priority": "5"
            },
            "payload": {
                "aps": {
                    "content-available": 1
                },
                "example": "example"
            }
        });
        assert_eq!(json, expect)
    }
}
