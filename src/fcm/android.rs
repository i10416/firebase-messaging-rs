use std::collections::HashMap;

use serde::Serialize;
/// In JSON format, the Duration type is encoded as a string rather than an object,
/// where the string ends in the suffix "s" (indicating seconds) and is preceded by
/// the number of seconds, with nanoseconds expressed as fractional seconds.
/// For example, 3 seconds with 0 nanoseconds should be encoded in JSON format as "3s",
/// while 3 seconds and 1 nanosecond should be expressed in JSON format as "3.000000001s".
/// Resolution defined by [proto.Duration](https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Duration)
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Duration(f32);
impl Duration {
    pub fn from_secs(secs: f32) -> Self {
        Self(secs)
    }
}
impl From<f32> for Duration {
    fn from(value: f32) -> Self {
        Self(value)
    }
}
impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{}s", self.0).serialize(serializer)
    }
}

/// Android specific options for messages sent through [FCM connection server](https://goo.gl/4GLdUl).
#[derive(Debug, Serialize, Default)]
pub struct AndroidConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Options for features provided by the FCM SDK for Android.
    pub fcm_options: Option<AndroidFcmOptions>,

    /// Message priority. Can take "normal" and "high" values. For more information, see [Setting the priority of a message](https://goo.gl/GjONJv).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<AndroidMessagePriority>,

    /// Notification to send to android devices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<AndroidNotification>,

    /// Arbitrary key/value payload. If present, it will override google.firebase.fcm.v1.Message.data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, String>>,

    /// Package name of the application where the registration token must match in order to receive the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restricted_package_name: Option<String>,

    /// How long (in seconds) the message should be kept in FCM storage if the device is offline.
    /// The maximum time to live supported is 4 weeks, and the default value is 4 weeks if not set.
    /// Set it to 0 if want to send the message immediately. In JSON format, the Duration type is
    /// encoded as a string rather than an object, where the string ends in the suffix "s" (indicating seconds)
    /// and is preceded by the number of seconds, with nanoseconds expressed as fractional seconds.
    /// For example, 3 seconds with 0 nanoseconds should be encoded in JSON format as "3s",
    /// while 3 seconds and 1 nanosecond should be expressed in JSON format as "3.000000001s".
    /// The ttl will be rounded down to the nearest second.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<Duration>,

    /// If set to true, messages will be allowed to be delivered to the app while the device is in direct boot mode. See [Support Direct Boot mode](https://developer.android.com/training/articles/direct-boot).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direct_boot_ok: Option<bool>,

    /// An identifier of a group of messages that can be collapsed, so that only the last message gets sent when delivery can be resumed. A maximum of 4 different collapse keys is allowed at any given time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapse_key: Option<String>,
}

/// Notification to send to android devices.
#[derive(Debug, Serialize, Default)]
pub struct AndroidNotification {
    /// Set whether or not this notification is relevant only to the current device.
    /// Some notifications can be bridged to other devices for remote display,
    /// such as a Wear OS watch. This hint can be set to recommend this notification
    /// not be bridged.
    /// See [Wear OS guides](https://developer.android.com/training/wearables/notifications/bridger#existing-method-of-preventing-bridging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_only: Option<bool>,

    /// If set to true, use the Android framework's default LED light settings for the
    /// notification. Default values are specified in
    /// [config.xml](https://android.googlesource.com/platform/frameworks/base/+/master/core/res/res/values/config.xml).
    /// If `default_light_settings` is set to true and `light_settings` is also set, the user-specified
    /// `light_settings` is used instead of the default value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_light_settings: Option<bool>,

    /// If set to true, use the Android framework's default sound for the notification.
    /// Default values are specified in
    /// [config.xml](https://android.googlesource.com/platform/frameworks/base/+/master/core/res/res/values/config.xml).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_sound: Option<bool>,

    /// Contains the URL of an image that is going to be displayed in a notification.
    /// If present, it will override google.firebase.fcm.v1.Notification.image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Identifier used to replace existing notifications in the notification drawer.
    /// If not specified, each request creates a new notification.
    /// If specified and a notification with the same tag is already being shown, the new notification
    /// replaces the existing one in the notification drawer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// If set to true, use the Android framework's default vibrate pattern for the notification.
    /// Default values are specified in
    /// [config.xml](https://android.googlesource.com/platform/frameworks/base/+/master/core/res/res/values/config.xml).
    /// If `default_vibrate_timings` is set to true and `vibrate_timings` is also set,
    /// the default value is used instead of the user-specified `vibrate_timings`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_vibrate_timings: Option<bool>,

    /// Sets the number of items this notification represents.
    /// May be displayed as a badge count for launchers that support badging.
    /// See [Notification Badge](https://developer.android.com/training/notify-user/badges).
    /// For example, this might be useful if you're using just one notification to represent
    /// multiple new messages but you want the count here to represent the number of total new messages.
    /// If zero or unspecified, systems that support badging use the default, which is to increment a number
    /// displayed on the long-press menu each time a new notification arrives.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_count: Option<u32>,

    /// The key to the title string in the app's string resources to use
    /// to localize the title text to the user's current localization.
    /// See [String Resources](https://goo.gl/NdFZGI) for more information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_loc_key: Option<String>,

    /// If set, display notifications delivered to the device will be
    /// handled by the app instead of the proxy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(since = "0.8.4")]
    pub bypass_proxy_notification: Option<bool>,

    /// The action associated with a user click on the notification.
    /// If specified, an activity with a matching intent filter is
    /// launched when a user clicks on the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_action: Option<String>,

    /// The sound to play when the device receives the notification.
    /// Supports "default" or the filename of a sound resource bundled
    /// in the app. Sound files must reside in /res/raw/.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,

    /// Set the time that the event in the notification occurred.
    /// Notifications in the panel are sorted by this time.
    /// A point in time is represented using
    /// [protobuf.Timestamp](https://developers.google.com/protocol-buffers/docs/reference/java/com/google/protobuf/Timestamp).
    ///
    /// Example: "2014-10-02T15:01:23Z", "2014-10-02T15:01:23.045123456Z"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_time: Option<String>,

    /// The notification's title. If present, it will override
    /// google.firebase.fcm.v1.Notification.title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Set the vibration pattern to use. Pass in an array of
    /// [protobuf.Duration](https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Duration)
    /// to turn on or off the vibrator.
    /// The first value indicates the `Duration` to wait before turning the vibrator on.
    /// The next value indicates the `Duration` to keep the vibrator on.
    /// Subsequent values alternate between `Duration` to turn the vibrator off and to turn the vibrator on.
    /// If `vibrate_timings` is set and `default_vibrate_timings` is set to `true`,
    /// the default value is used instead of the user-specified `vibrate_timings`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vibrate_timings: Option<Vec<Duration>>,

    /// The key to the body string in the app's string resources to use
    /// to localize the body text to the user's current localization.
    /// See [String Resources](https://goo.gl/NdFZGI) for more information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_loc_key: Option<String>,

    /// The notification's body text. If present, it will override
    /// google.firebase.fcm.v1.Notification.body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// The notification's icon. Sets the notification icon to myicon
    /// for drawable resource myicon. If you don't send this key in the request,
    /// FCM displays the launcher icon specified in your app manifest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// Variable string values to be used in place of the format
    /// specifiers in title_loc_key to use to localize the title text to
    /// the user's current localization.
    /// See [Formatting and Styling](https://goo.gl/MalYE3) for more information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_loc_args: Option<Vec<String>>,

    /// The notification's icon color, expressed in #rrggbb format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Variable string values to be used in place of the format
    /// specifiers in body_loc_key to use to localize the body text
    /// to the user's current localization.
    /// See [Formatting and Styling](https://goo.gl/MalYE3) for more information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_loc_args: Option<Vec<String>>,

    /// When set to false or unset, the notification is
    /// automatically dismissed when the user clicks it in the panel.
    /// When set to true, the notification persists even when the user clicks it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sticky: Option<bool>,

    /// Setting to control when a notification may be proxied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<Proxy>,

    /// Sets the "ticker" text, which is sent to accessibility services.
    /// Prior to API level 21 (`Lollipop`), sets the text that is displayed
    /// in the status bar when the notification first arrives.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    /// Set the relative priority for this notification.
    /// Priority is an indication of how much of the user's attention
    /// should be consumed by this notification.
    /// Low-priority notifications may be hidden from the user in certain situations,
    /// while the user might be interrupted for a higher-priority notification.
    /// The effect of setting the same priorities may differ slightly on different platforms.
    /// Note this priority differs from `AndroidMessagePriority`.
    /// This priority is processed by the client after the message has been delivered, whereas [AndroidMessagePriority](https://firebase.google.com/docs/reference/fcm/rest/v1/projects.messages#androidmessagepriority) is an FCM concept that controls when the message is delivered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_priority: Option<NotificationPriority>,

    /// Set the [Notification.visibility](https://developer.android.com/reference/android/app/Notification.html#visibility)
    /// of the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,

    /// The [notification's channel id](https://developer.android.com/guide/topics/ui/notifiers/notifications#ManageChannels)(new in Android O).
    /// The app must create a channel with this channel ID before any notification with this channel ID is received.
    /// If you don't send this channel ID in the request, or if the channel ID provided has not yet been created by the app,
    /// FCM uses the channel ID specified in the app manifest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,

    /// Settings to control the notification's LED blinking rate and color if LED is available on the device.
    /// The total blinking time is controlled by the OS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_settings: Option<LightSettings>,
}

/// Settings to control notification LED.
#[derive(Debug, Serialize, Default)]
pub struct LightSettings {
    pub color: Color,
    /// Along with `light_off_duration`, define the blink rate of LED flashes.
    /// Resolution defined by [proto.Duration](https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Duration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_on_duration: Option<Duration>,
    /// Along with `light_on_duration `, define the blink rate of LED flashes.
    /// Resolution defined by [proto.Duration](https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Duration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_off_duration: Option<Duration>,
}

/// Set `color` of the LED with [google.type.Color](https://github.com/googleapis/googleapis/blob/master/google/type/color.proto).
#[derive(Debug, Serialize, Default)]
pub struct Color {
    /// The amount of red in the color as a value in the interval [0, 1].
    pub red: f32,
    /// The amount of green in the color as a value in the interval [0, 1].
    pub green: f32,
    /// The amount of blue in the color as a value in the interval [0, 1].
    pub blue: f32,
    /// The fraction of this color that should be applied to the pixel. That is, the final pixel color is defined
    /// by the equation: `pixel color = alpha * (this color) + (1.0 - alpha) * (background color)`
    /// This means that a value of 1.0 corresponds to a solid color, whereas a
    /// value of 0.0 corresponds to a completely transparent color.
    /// This uses a wrapper message rather than a simple float scalar so that
    /// it is possible to distinguish between a default value and the value being unset.
    /// If omitted, this color object is rendered as a solid color
    /// (as if the alpha value had been explicitly given a value of 1.0).
    pub alpha: f32,
}

#[derive(Debug, Serialize)]
pub enum Proxy {
    #[serde(rename = "PROXY_UNSPECIFIED")]
    ProxyUnspecified,

    /// Try to proxy this notification.
    #[serde(rename = "ALLOW")]
    Allow,

    /// Do not proxy this notification.
    #[serde(rename = "DENY")]
    Deny,

    /// Only try to proxy this notification if its `AndroidMessagePriority` was lowered from `HIGH` to `NORMAL` on the device.
    #[serde(rename = "IF_PRIORITY_LOWERED")]
    IfPriorityLowered,
}

impl Default for Proxy {
    fn default() -> Self {
        Self::IfPriorityLowered
    }
}

#[derive(Debug, Serialize)]
pub enum NotificationPriority {
    /// If priority is unspecified, notification priority is set to `PRIORITY_DEFAULT`.
    #[serde(rename = "PRIORITY_UNSPECIFIED")]
    PriorityUnspecified,

    /// Lowest notification priority. Notifications with this `PRIORITY_MIN` might not be shown to the user except under special circumstances, such as detailed notification logs.
    #[serde(rename = "PRIORITY_MIN")]
    PriorityMin,

    /// Lower notification priority. The UI may choose to show the notifications smaller, or at a different position in the list, compared with notifications with `PRIORITY_DEFAULT`.
    #[serde(rename = "PRIORITY_LOW")]
    PriorityLow,

    /// Default notification priority. If the application does not prioritize its own notifications, use this value for all notifications.
    #[serde(rename = "PRIORITY_DEFAULT")]
    PriorityDefault,

    /// Higher notification priority. Use this for more important notifications or alerts. The UI may choose to show these notifications larger, or at a different position in the notification lists, compared with notifications with `PRIORITY_DEFAULT`.
    #[serde(rename = "PRIORITY_HIGH")]
    PriorityHigh,

    /// Highest notification priority. Use this for the application's most important items that require the user's prompt attention or input.
    #[serde(rename = "PRIORITY_MAX")]
    PriorityMax,
}

impl Default for NotificationPriority {
    fn default() -> Self {
        Self::PriorityDefault
    }
}

#[derive(Debug, Serialize)]
pub enum Visibility {
    /// If unspecified, default to `Visibility.PRIVATE`.
    #[serde(rename = "VISIBILITY_UNSPECIFIED")]
    VisibilityUnspecified,

    /// Show this notification on all lockscreens, but conceal sensitive or private information on secure lockscreens.
    #[serde(rename = "PRIVATE")]
    Private,

    /// Show this notification in its entirety on all lockscreens.
    #[serde(rename = "PUBLIC")]
    Public,

    /// Do not reveal any part of this notification on a secure lockscreen.
    #[serde(rename = "SECRET")]
    Secret,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Private
    }
}

/// Message priority. Can take "normal" and "high" values.
/// For more information, see [Setting the priority of a message](https://goo.gl/GjONJv).
#[derive(Debug, Serialize)]
pub enum AndroidMessagePriority {
    /// Default priority for notification messages.
    /// FCM attempts to deliver high priority messages immediately,
    /// allowing the FCM service to wake a sleeping device when possible
    /// and open a network connection to your app server.
    /// Apps with instant messaging, chat, or voice call alerts,
    /// for example, generally need to open a network connection and make
    /// sure FCM delivers the message to the device without delay.
    /// Set high priority if the message is time-critical and requires
    /// the user's immediate interaction, but beware that setting your
    /// messages to high priority contributes more to battery drain compared
    /// with normal priority messages.
    #[serde(rename = "HIGH")]
    High,
    /// Default priority for data messages. Normal priority messages won't
    /// open network connections on a sleeping device, and their delivery
    /// may be delayed to conserve the battery. For less time-sensitive messages,
    /// such as notifications of new email or other data to sync, choose normal delivery
    /// priority.
    #[serde(rename = "NORMAL")]
    Normal,
}

impl Default for AndroidMessagePriority {
    fn default() -> Self {
        Self::High
    }
}

/// Options for features provided by the FCM SDK for Android.
#[derive(Debug, Serialize, Default)]
pub struct AndroidFcmOptions {
    /// Label associated with the message's analytics data.
    #[serde(skip_serializing_if = "Option::is_none")]
    analytics_label: Option<String>,
}

impl AndroidFcmOptions {
    pub fn new(analytics_label: &str) -> Self {
        Self {
            analytics_label: Some(analytics_label.to_string()),
        }
    }
}
