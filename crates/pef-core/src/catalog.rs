//! The built-in catalog of Windows privacy policies and the presets that group them.
//!
//! Every entry maps one documented Windows policy to a single `DWORD` registry value below
//! `HKEY_LOCAL_MACHINE`. The catalog is the only source of registry paths: backup files
//! carry policy IDs, never paths, so a tampered backup cannot redirect a write. Nothing in
//! the catalog disables Microsoft Defender, Windows Update, User Account Control, event
//! logging, authentication or network connectivity.

use serde::Serialize;

use crate::error::Error;

/// How disruptive a policy can be for everyday use of the machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub enum ImpactLevel {
    /// Changes a background behaviour; applications keep working.
    Moderate,
    /// Can break calls, recording, dictation, notifications or location features until reverted.
    High,
}

impl ImpactLevel {
    /// A short label for user interfaces.
    pub const fn label(self) -> &'static str {
        match self {
            ImpactLevel::Moderate => "Moderate impact",
            ImpactLevel::High => "High impact",
        }
    }
}

/// The area of Windows a policy belongs to. Used to group the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub enum Category {
    /// Diagnostic data, crash reports and usage programs sent to Microsoft.
    DiagnosticData,
    /// Advertising identifiers, suggestions and promotional content.
    Personalization,
    /// Windows Search, Cortana and web results.
    Search,
    /// Typing, inking, handwriting and speech data.
    InputAndSpeech,
    /// The device location platform.
    Location,
    /// Permissions of packaged Windows apps.
    AppPermissions,
    /// Activity history, clipboard, settings and phone sync.
    ActivityAndSync,
    /// Background downloads and peer-to-peer services.
    NetworkServices,
    /// Recall, Click to Do and other on-device AI features.
    WindowsAi,
    /// Microsoft Edge browser reporting.
    Edge,
}

impl Category {
    /// Every category in display order.
    pub const ALL: [Category; 10] = [
        Category::DiagnosticData,
        Category::Personalization,
        Category::Search,
        Category::InputAndSpeech,
        Category::Location,
        Category::AppPermissions,
        Category::ActivityAndSync,
        Category::NetworkServices,
        Category::WindowsAi,
        Category::Edge,
    ];

    /// Short title.
    pub const fn label(self) -> &'static str {
        match self {
            Category::DiagnosticData => "Diagnostic data",
            Category::Personalization => "Personalization & ads",
            Category::Search => "Search",
            Category::InputAndSpeech => "Input & speech",
            Category::Location => "Location",
            Category::AppPermissions => "App permissions",
            Category::ActivityAndSync => "Activity & sync",
            Category::NetworkServices => "Network services",
            Category::WindowsAi => "Windows AI",
            Category::Edge => "Microsoft Edge",
        }
    }

    /// One sentence describing the group.
    pub const fn description(self) -> &'static str {
        match self {
            Category::DiagnosticData => {
                "Usage, crash and compatibility data that Windows sends to Microsoft."
            }
            Category::Personalization => {
                "Advertising identifiers, suggested content and promotional experiences."
            }
            Category::Search => "What the Search box sends to the web and the cloud.",
            Category::InputAndSpeech => {
                "Collection of typing, inking, handwriting and voice data for personalization."
            }
            Category::Location => "The device-wide location platform and Find My Device.",
            Category::AppPermissions => {
                "Force-denies a capability for packaged Windows apps. Desktop apps are not covered."
            }
            Category::ActivityAndSync => {
                "Activity history, clipboard history, settings roaming and cross-device features."
            }
            Category::NetworkServices => {
                "Background connections such as font downloads and peer-to-peer update sharing."
            }
            Category::WindowsAi => "Recall snapshots and on-device AI experiences.",
            Category::Edge => {
                "Reporting from Microsoft Edge. Edge then shows a managed-by-organization notice."
            }
        }
    }
}

/// One Windows policy the tool knows how to audit, enforce and restore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PrivacyPolicy {
    /// Stable identifier used on the command line and inside backup files.
    pub id: &'static str,
    /// Short human readable name.
    pub title: &'static str,
    /// The area of Windows the policy belongs to.
    pub category: Category,
    /// Registry key path below `HKEY_LOCAL_MACHINE`, read through the 64-bit view.
    pub key: &'static str,
    /// Registry value name inside `key`.
    pub name: &'static str,
    /// The `DWORD` value that enforces the policy.
    pub desired: u32,
    /// What enabling the policy does and does not do.
    pub impact: &'static str,
    /// How disruptive enforcement can be.
    pub impact_level: ImpactLevel,
    /// Microsoft reference for the policy.
    pub source: &'static str,
}

impl PrivacyPolicy {
    /// The full registry location in the form shown by Registry Editor.
    pub fn registry_path(&self) -> String {
        format!("HKLM\\{}\\{}", self.key, self.name)
    }
}

/// A named shorthand for an explicit policy selection.
///
/// Presets never bypass the preview: expanding one produces the same ID list a person
/// could have typed, and the preview shows every ID before anything is confirmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Preset {
    /// Stable identifier used with `--preset`.
    pub id: &'static str,
    /// Short human readable name.
    pub title: &'static str,
    /// What the preset covers and what it leaves alone.
    pub description: &'static str,
}

const DATA_COLLECTION: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection";
const CLOUD_CONTENT: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\CloudContent";
const WINDOWS_SEARCH: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\Windows Search";
const INPUT_PERSONALIZATION: &str = "SOFTWARE\\Policies\\Microsoft\\InputPersonalization";
const APP_PRIVACY: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\AppPrivacy";
const SYSTEM: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\System";
const WINDOWS_AI: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsAI";
const EDGE: &str = "SOFTWARE\\Policies\\Microsoft\\Edge";
const APP_COMPAT: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\AppCompat";

const MANAGE_CONNECTIONS: &str = "https://learn.microsoft.com/en-us/windows/privacy/manage-connections-from-windows-operating-system-components-to-microsoft-services";

macro_rules! csp {
    ($area:literal, $anchor:literal) => {
        concat!(
            "https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-",
            $area,
            "#",
            $anchor
        )
    };
}

macro_rules! edge_policy {
    ($anchor:literal) => {
        concat!(
            "https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#",
            $anchor
        )
    };
}

#[allow(clippy::too_many_arguments)]
const fn policy(
    id: &'static str,
    title: &'static str,
    category: Category,
    key: &'static str,
    name: &'static str,
    desired: u32,
    impact_level: ImpactLevel,
    impact: &'static str,
    source: &'static str,
) -> PrivacyPolicy {
    PrivacyPolicy {
        id,
        title,
        category,
        key,
        name,
        desired,
        impact,
        impact_level,
        source,
    }
}

use Category::*;
use ImpactLevel::*;

static ALL: [PrivacyPolicy; 65] = [
    // ------------------------------------------------------------------ diagnostic data
    policy(
        "diagnostic-data",
        "Diagnostic data level",
        DiagnosticData,
        DATA_COLLECTION,
        "AllowTelemetry",
        0,
        Moderate,
        "Sets diagnostic data to the lowest level the edition supports. Pro treats 0 as Required; only Enterprise, Education and Server can turn diagnostic data off. Does not stop every connection to Microsoft.",
        csp!("system", "allowtelemetry"),
    ),
    policy(
        "diagnostic-log-collection",
        "Additional diagnostic logs",
        DiagnosticData,
        DATA_COLLECTION,
        "LimitDiagnosticLogCollection",
        1,
        Moderate,
        "Stops Windows from collecting additional diagnostic logs when Microsoft requests them.",
        csp!("system", "limitdiagnosticlogcollection"),
    ),
    policy(
        "crash-dump-collection",
        "Crash dump upload",
        DiagnosticData,
        DATA_COLLECTION,
        "LimitDumpCollection",
        1,
        Moderate,
        "Limits crash dumps included in diagnostic data to the smallest type; local crash dumps and event logs are unaffected.",
        csp!("system", "limitdumpcollection"),
    ),
    policy(
        "onesettings-downloads",
        "OneSettings configuration downloads",
        DiagnosticData,
        DATA_COLLECTION,
        "DisableOneSettingsDownloads",
        1,
        Moderate,
        "Stops the diagnostic data service from downloading configuration from the OneSettings service.",
        csp!("system", "disableonesettingsdownloads"),
    ),
    policy(
        "device-name-in-diagnostics",
        "Device name in diagnostic data",
        DiagnosticData,
        DATA_COLLECTION,
        "AllowDeviceNameInTelemetry",
        0,
        Moderate,
        "Removes the computer name from diagnostic data sent to Microsoft.",
        csp!("system", "allowdevicenameindiagnosticdata"),
    ),
    policy(
        "feedback-notifications",
        "Feedback requests",
        DiagnosticData,
        DATA_COLLECTION,
        "DoNotShowFeedbackNotifications",
        1,
        Moderate,
        "Stops Windows from asking for feedback; the Feedback Hub still opens on request.",
        csp!("experience", "donotshowfeedbacknotifications"),
    ),
    policy(
        "ceip",
        "Customer Experience Improvement Program",
        DiagnosticData,
        "SOFTWARE\\Policies\\Microsoft\\SQMClient\\Windows",
        "CEIPEnable",
        0,
        Moderate,
        "Turns off the Windows Customer Experience Improvement Program for all users.",
        MANAGE_CONNECTIONS,
    ),
    policy(
        "error-reporting",
        "Windows Error Reporting upload",
        DiagnosticData,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\Windows Error Reporting",
        "Disabled",
        1,
        Moderate,
        "Stops crash and hang reports being sent to Microsoft; applications still close normally and local event logging continues.",
        csp!("errorreporting", "disablewindowserrorreporting"),
    ),
    policy(
        "app-compat-inventory",
        "Application compatibility inventory",
        DiagnosticData,
        APP_COMPAT,
        "DisableInventory",
        1,
        Moderate,
        "Turns off the Inventory Collector that reports installed programs, devices and drivers to Microsoft.",
        MANAGE_CONNECTIONS,
    ),
    policy(
        "app-telemetry",
        "Application telemetry",
        DiagnosticData,
        APP_COMPAT,
        "AITEnable",
        0,
        Moderate,
        "Turns off Application Telemetry, which tracks program usage for compatibility reporting.",
        MANAGE_CONNECTIONS,
    ),
    // ------------------------------------------------------------------ personalization
    policy(
        "advertising-id",
        "Advertising ID",
        Personalization,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\AdvertisingInfo",
        "DisabledByGroupPolicy",
        1,
        Moderate,
        "Stops apps using the Windows advertising ID; does not block all advertising or tracking.",
        csp!("privacy", "disableadvertisingid"),
    ),
    policy(
        "consumer-features",
        "Consumer experiences",
        Personalization,
        CLOUD_CONTENT,
        "DisableWindowsConsumerFeatures",
        1,
        Moderate,
        "Stops suggested apps and Microsoft account promotions. Microsoft documents it for Enterprise and Education; Pro may ignore it.",
        csp!("experience", "allowwindowsconsumerfeatures"),
    ),
    policy(
        "cloud-optimized-content",
        "Cloud optimized content",
        Personalization,
        CLOUD_CONTENT,
        "DisableCloudOptimizedContent",
        1,
        Moderate,
        "Turns off cloud-delivered content such as taskbar and Start suggestions in Windows experiences.",
        csp!("experience", "disablecloudoptimizedcontent"),
    ),
    policy(
        "windows-tips",
        "Windows tips",
        Personalization,
        CLOUD_CONTENT,
        "DisableSoftLanding",
        1,
        Moderate,
        "Stops Windows tips and suggestions. Microsoft documents it for Enterprise and Education; Pro may ignore it.",
        csp!("experience", "allowwindowstips"),
    ),
    // ------------------------------------------------------------------ search
    policy(
        "cortana",
        "Cortana",
        Search,
        WINDOWS_SEARCH,
        "AllowCortana",
        0,
        Moderate,
        "Disables Cortana. Windows 11 has retired the Cortana app, so this mostly affects older builds.",
        csp!("experience", "allowcortana"),
    ),
    policy(
        "search-web-results",
        "Web results in Search",
        Search,
        WINDOWS_SEARCH,
        "ConnectedSearchUseWeb",
        0,
        Moderate,
        "Stops the Search box from querying the web; local search still works. Microsoft documents it for Enterprise and Education; Pro may ignore it.",
        csp!("search", "donotusewebresults"),
    ),
    policy(
        "search-location",
        "Search location use",
        Search,
        WINDOWS_SEARCH,
        "AllowSearchToUseLocation",
        0,
        Moderate,
        "Stops Search and Cortana from using the device location.",
        csp!("search", "allowsearchtouselocation"),
    ),
    policy(
        "cloud-search",
        "Cloud search",
        Search,
        WINDOWS_SEARCH,
        "AllowCloudSearch",
        0,
        Moderate,
        "Stops Search from querying cloud content for work or school accounts.",
        csp!("search", "allowcloudsearch"),
    ),
    policy(
        "search-highlights",
        "Search highlights",
        Search,
        WINDOWS_SEARCH,
        "EnableDynamicContentInWSB",
        0,
        Moderate,
        "Turns off search highlights, the dynamic content shown in the Search box.",
        csp!("search", "allowsearchhighlights"),
    ),
    // ------------------------------------------------------------------ input & speech
    policy(
        "online-speech",
        "Online speech recognition",
        InputAndSpeech,
        INPUT_PERSONALIZATION,
        "AllowInputPersonalization",
        0,
        High,
        "Disables cloud speech recognition and may prevent voice dictation.",
        csp!("privacy", "allowinputpersonalization"),
    ),
    policy(
        "typing-data-collection",
        "Typing data collection",
        InputAndSpeech,
        INPUT_PERSONALIZATION,
        "RestrictImplicitTextCollection",
        1,
        Moderate,
        "Stops typing data being collected for personalization; typing suggestions may become generic.",
        csp!("textinput", "allowlinguisticdatacollection"),
    ),
    policy(
        "inking-data-collection",
        "Inking data collection",
        InputAndSpeech,
        INPUT_PERSONALIZATION,
        "RestrictImplicitInkCollection",
        1,
        Moderate,
        "Stops inking data being collected for handwriting personalization.",
        csp!("textinput", "allowlinguisticdatacollection"),
    ),
    policy(
        "speech-model-updates",
        "Speech model updates",
        InputAndSpeech,
        "SOFTWARE\\Policies\\Microsoft\\Speech",
        "AllowSpeechModelUpdate",
        0,
        Moderate,
        "Stops automatic download of updated speech recognition and synthesis models.",
        csp!("speech", "allowspeechmodelupdate"),
    ),
    policy(
        "handwriting-error-reports",
        "Handwriting error reports",
        InputAndSpeech,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\HandwritingErrorReports",
        "PreventHandwritingErrorReports",
        1,
        Moderate,
        "Stops handwriting recognition errors being reported to Microsoft.",
        MANAGE_CONNECTIONS,
    ),
    // ------------------------------------------------------------------ location
    policy(
        "location-platform",
        "Location platform",
        Location,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\LocationAndSensors",
        "DisableLocation",
        1,
        High,
        "Turns off location for the whole device: maps, weather, time zone detection and Find My Device stop working until reverted.",
        csp!("system", "allowlocation"),
    ),
    policy(
        "find-my-device",
        "Find My Device",
        Location,
        "SOFTWARE\\Policies\\Microsoft\\FindMyDevice",
        "AllowFindMyDevice",
        0,
        Moderate,
        "Turns off Find My Device; a lost device can no longer be located or locked from the Microsoft account.",
        csp!("experience", "allowfindmydevice"),
    ),
    // ------------------------------------------------------------------ app permissions
    policy(
        "app-camera",
        "App camera access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessCamera",
        2,
        High,
        "Denies camera access to covered Windows apps; video calls can break. Per-app policy overrides and desktop apps need separate review.",
        csp!("privacy", "letappsaccesscamera"),
    ),
    policy(
        "app-microphone",
        "App microphone access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessMicrophone",
        2,
        High,
        "Denies microphone access to covered Windows apps; calls and recording can break. Per-app policy overrides and desktop apps need separate review.",
        csp!("privacy", "letappsaccessmicrophone"),
    ),
    policy(
        "app-location",
        "App location access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessLocation",
        2,
        High,
        "Denies location access to covered Windows apps; maps and location features can break. Per-app policy overrides and desktop apps need separate review.",
        csp!("privacy", "letappsaccesslocation"),
    ),
    policy(
        "app-account-info",
        "App account info access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessAccountInfo",
        2,
        Moderate,
        "Denies covered Windows apps access to the account name, picture and other account information; some apps may need manual sign-in.",
        csp!("privacy", "letappsaccessaccountinfo"),
    ),
    policy(
        "app-contacts",
        "App contacts access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessContacts",
        2,
        Moderate,
        "Denies covered Windows apps access to contacts.",
        csp!("privacy", "letappsaccesscontacts"),
    ),
    policy(
        "app-calendar",
        "App calendar access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessCalendar",
        2,
        Moderate,
        "Denies covered Windows apps access to the calendar.",
        csp!("privacy", "letappsaccesscalendar"),
    ),
    policy(
        "app-call-history",
        "App call history access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessCallHistory",
        2,
        Moderate,
        "Denies covered Windows apps access to call history.",
        csp!("privacy", "letappsaccesscallhistory"),
    ),
    policy(
        "app-email",
        "App email access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessEmail",
        2,
        Moderate,
        "Denies covered Windows apps access to email.",
        csp!("privacy", "letappsaccessemail"),
    ),
    policy(
        "app-messaging",
        "App messaging access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessMessaging",
        2,
        Moderate,
        "Denies covered Windows apps access to text and MMS messages.",
        csp!("privacy", "letappsaccessmessaging"),
    ),
    policy(
        "app-phone",
        "App phone call access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessPhone",
        2,
        Moderate,
        "Denies covered Windows apps the ability to make phone calls.",
        csp!("privacy", "letappsaccessphone"),
    ),
    policy(
        "app-radios",
        "App radio control",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessRadios",
        2,
        Moderate,
        "Denies covered Windows apps control of radios such as Bluetooth and Wi-Fi.",
        csp!("privacy", "letappsaccessradios"),
    ),
    policy(
        "app-motion",
        "App motion data access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessMotion",
        2,
        Moderate,
        "Denies covered Windows apps access to motion and activity sensors.",
        csp!("privacy", "letappsaccessmotion"),
    ),
    policy(
        "app-notifications",
        "App notification access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessNotifications",
        2,
        Moderate,
        "Denies covered Windows apps access to the notifications of other apps.",
        csp!("privacy", "letappsaccessnotifications"),
    ),
    policy(
        "app-tasks",
        "App tasks access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessTasks",
        2,
        Moderate,
        "Denies covered Windows apps access to tasks.",
        csp!("privacy", "letappsaccesstasks"),
    ),
    policy(
        "app-diagnostics",
        "App diagnostic info access",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsGetDiagnosticInfo",
        2,
        Moderate,
        "Denies covered Windows apps access to diagnostic information about other apps.",
        csp!("privacy", "letappsgetdiagnosticinfo"),
    ),
    policy(
        "app-voice-activation",
        "App voice activation",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsActivateWithVoice",
        2,
        Moderate,
        "Stops covered Windows apps from activating with a voice keyword.",
        csp!("privacy", "letappsactivatewithvoice"),
    ),
    policy(
        "app-voice-activation-locked",
        "App voice activation above lock",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsActivateWithVoiceAboveLock",
        2,
        Moderate,
        "Stops covered Windows apps from activating with a voice keyword while the device is locked.",
        csp!("privacy", "letappsactivatewithvoiceabovelock"),
    ),
    policy(
        "app-background",
        "Background apps",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsRunInBackground",
        2,
        High,
        "Stops covered Windows apps running in the background: notifications, mail sync and live updates from those apps stop.",
        csp!("privacy", "letappsruninbackground"),
    ),
    policy(
        "app-screen-capture",
        "App programmatic screen capture",
        AppPermissions,
        APP_PRIVACY,
        "LetAppsAccessGraphicsCaptureProgrammatic",
        2,
        High,
        "Denies covered Windows apps programmatic screen capture; screen sharing in those apps can break.",
        csp!("privacy", "letappsaccessgraphicscaptureprogrammatic"),
    ),
    // ------------------------------------------------------------------ activity & sync
    policy(
        "activity-feed",
        "Activity feed",
        ActivityAndSync,
        SYSTEM,
        "EnableActivityFeed",
        0,
        Moderate,
        "Restricts activity feed publishing and roaming; does not erase previously collected data.",
        csp!("privacy", "enableactivityfeed"),
    ),
    policy(
        "activity-publish",
        "Activity publishing",
        ActivityAndSync,
        SYSTEM,
        "PublishUserActivities",
        0,
        Moderate,
        "Stops apps from publishing user activities to the activity feed.",
        csp!("privacy", "publishuseractivities"),
    ),
    policy(
        "activity-upload",
        "Activity upload",
        ActivityAndSync,
        SYSTEM,
        "UploadUserActivities",
        0,
        Moderate,
        "Stops user activities being uploaded to the Microsoft cloud.",
        csp!("privacy", "uploaduseractivities"),
    ),
    policy(
        "clipboard-history",
        "Clipboard history",
        ActivityAndSync,
        SYSTEM,
        "AllowClipboardHistory",
        0,
        Moderate,
        "Turns off clipboard history (Win+V); ordinary copy and paste keeps working.",
        csp!("privacy", "allowclipboardhistory"),
    ),
    policy(
        "clipboard-sync",
        "Cross-device clipboard",
        ActivityAndSync,
        SYSTEM,
        "AllowCrossDeviceClipboard",
        0,
        Moderate,
        "Prevents clipboard synchronization to other devices; local copy/paste remains available.",
        csp!("privacy", "allowcrossdeviceclipboard"),
    ),
    policy(
        "settings-sync",
        "Settings sync",
        ActivityAndSync,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\SettingSync",
        "DisableSettingSync",
        2,
        Moderate,
        "Stops settings, passwords and preferences roaming to other devices through the Microsoft account.",
        csp!("experience", "allowsyncmysettings"),
    ),
    policy(
        "settings-sync-user-override",
        "Settings sync user override",
        ActivityAndSync,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\SettingSync",
        "DisableSettingSyncUserOverride",
        1,
        Moderate,
        "Prevents users from re-enabling settings sync while the settings-sync policy is enforced.",
        csp!("experience", "allowsyncmysettings"),
    ),
    policy(
        "message-sync",
        "Text message cloud sync",
        ActivityAndSync,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\Messaging",
        "AllowMessageSync",
        0,
        Moderate,
        "Stops text messages being backed up and synchronized through the Microsoft cloud.",
        csp!("messaging", "allowmessagesync"),
    ),
    policy(
        "phone-link",
        "Phone Link",
        ActivityAndSync,
        SYSTEM,
        "EnableMmx",
        0,
        Moderate,
        "Turns off Phone Link; the phone can no longer be paired with this PC.",
        csp!("connectivity", "allowphonepclinking"),
    ),
    policy(
        "cloud-notifications",
        "Cloud push notifications",
        ActivityAndSync,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\CurrentVersion\\PushNotifications",
        "NoCloudApplicationNotification",
        1,
        High,
        "Stops push notifications delivered through the Windows notification cloud service; apps such as Mail and Teams lose live notifications.",
        csp!("notifications", "disallowcloudnotification"),
    ),
    // ------------------------------------------------------------------ network services
    policy(
        "font-providers",
        "Online font providers",
        NetworkServices,
        SYSTEM,
        "EnableFontProviders",
        0,
        Moderate,
        "Stops Windows from downloading fonts that are not installed locally; documents may fall back to substitute fonts.",
        csp!("system", "allowfontproviders"),
    ),
    policy(
        "delivery-optimization-p2p",
        "Update sharing with other PCs",
        NetworkServices,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\DeliveryOptimization",
        "DODownloadMode",
        0,
        Moderate,
        "Sets Delivery Optimization to HTTP only: updates are neither shared with nor fetched from other PCs. Windows Update keeps working.",
        csp!("deliveryoptimization", "dodownloadmode"),
    ),
    policy(
        "device-metadata",
        "Device metadata downloads",
        NetworkServices,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\Device Metadata",
        "PreventDeviceMetadataFromNetwork",
        1,
        Moderate,
        "Stops Windows from downloading device icons and descriptions for attached hardware from the internet.",
        MANAGE_CONNECTIONS,
    ),
    // ------------------------------------------------------------------ windows ai
    policy(
        "recall-snapshots",
        "Recall snapshots",
        WindowsAi,
        WINDOWS_AI,
        "DisableAIDataAnalysis",
        1,
        Moderate,
        "Stops Recall from saving snapshots of the screen; existing snapshots are not deleted by this policy.",
        csp!("windowsai", "disableaidataanalysis"),
    ),
    policy(
        "recall-enablement",
        "Recall component",
        WindowsAi,
        WINDOWS_AI,
        "AllowRecallEnablement",
        0,
        Moderate,
        "Prevents Recall from being enabled on Copilot+ PCs; takes effect after a restart.",
        csp!("windowsai", "allowrecallenablement"),
    ),
    policy(
        "click-to-do",
        "Click to Do",
        WindowsAi,
        WINDOWS_AI,
        "DisableClickToDo",
        1,
        Moderate,
        "Turns off Click to Do, the on-screen content analysis on Copilot+ PCs.",
        csp!("windowsai", "disableclicktodo"),
    ),
    // ------------------------------------------------------------------ microsoft edge
    policy(
        "edge-personalization-reporting",
        "Edge personalization reporting",
        Edge,
        EDGE,
        "PersonalizationReportingEnabled",
        0,
        Moderate,
        "Stops Microsoft Edge from sending browsing history to Microsoft for personalized advertising and content.",
        edge_policy!("personalizationreportingenabled"),
    ),
    policy(
        "edge-diagnostic-data",
        "Edge diagnostic data",
        Edge,
        EDGE,
        "DiagnosticData",
        0,
        Moderate,
        "Turns off required and optional diagnostic data from Microsoft Edge.",
        edge_policy!("diagnosticdata"),
    ),
    policy(
        "edge-search-suggestions",
        "Edge search suggestions",
        Edge,
        EDGE,
        "SearchSuggestEnabled",
        0,
        Moderate,
        "Stops typed address bar text being sent to the search provider for suggestions.",
        edge_policy!("searchsuggestenabled"),
    ),
    policy(
        "edge-shopping-assistant",
        "Edge shopping assistant",
        Edge,
        EDGE,
        "EdgeShoppingAssistantEnabled",
        0,
        Moderate,
        "Turns off the Microsoft Edge shopping assistant, which sends page data to Microsoft for coupons and price comparison.",
        edge_policy!("edgeshoppingassistantenabled"),
    ),
];

static PRESETS: [Preset; 4] = [
    Preset {
        id: "baseline",
        title: "Baseline",
        description: "Every moderate-impact Windows policy outside app permissions and Microsoft Edge: diagnostic data, ads, search, input, sync, network services and Windows AI. Applications keep working.",
    },
    Preset {
        id: "app-permissions",
        title: "App permissions",
        description: "Force-denies every covered capability for packaged Windows apps, including camera, microphone, location and background activity. Expect apps to break until reverted.",
    },
    Preset {
        id: "edge",
        title: "Microsoft Edge",
        description: "Every Microsoft Edge reporting policy. Edge shows a managed-by-organization notice afterwards.",
    },
    Preset {
        id: "strict",
        title: "Strict",
        description: "Every policy in the catalog, including the high-impact location, notification, speech and app-permission policies. Review each impact first.",
    },
];

/// Every policy in the catalog, in display order.
pub fn all() -> &'static [PrivacyPolicy] {
    &ALL
}

/// Every policy of one category, in display order.
pub fn in_category(category: Category) -> impl Iterator<Item = &'static PrivacyPolicy> {
    ALL.iter().filter(move |policy| policy.category == category)
}

/// Looks up one policy by ID. IDs are case sensitive.
pub fn get(id: &str) -> Result<&'static PrivacyPolicy, Error> {
    ALL.iter()
        .find(|policy| policy.id == id)
        .ok_or_else(|| Error::UnknownPolicy(id.to_owned()))
}

/// Resolves an explicit, non-empty selection of policy IDs.
///
/// Duplicates are removed while preserving the first occurrence. An unknown ID or an empty
/// selection is an error: there is deliberately no implicit "everything" for enforcement.
pub fn select<I, S>(ids: I) -> Result<Vec<&'static PrivacyPolicy>, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut selected: Vec<&'static PrivacyPolicy> = Vec::new();
    for id in ids {
        let policy = get(id.as_ref())?;
        if !selected.iter().any(|existing| existing.id == policy.id) {
            selected.push(policy);
        }
    }
    if selected.is_empty() {
        return Err(Error::EmptySelection);
    }
    Ok(selected)
}

/// Returns true when the policy belongs to the built-in catalog, field for field.
pub fn contains(policy: &PrivacyPolicy) -> bool {
    get(policy.id).is_ok_and(|known| known == policy)
}

/// Every preset, in display order.
pub fn presets() -> &'static [Preset] {
    &PRESETS
}

/// Looks up one preset by ID.
pub fn preset(id: &str) -> Result<&'static Preset, Error> {
    PRESETS
        .iter()
        .find(|preset| preset.id == id)
        .ok_or_else(|| Error::UnknownPreset(id.to_owned()))
}

/// The policies a preset stands for, in catalog order.
pub fn expand_preset(preset: &Preset) -> Vec<&'static PrivacyPolicy> {
    ALL.iter()
        .filter(|policy| match preset.id {
            "baseline" => {
                policy.impact_level == Moderate
                    && policy.category != AppPermissions
                    && policy.category != Edge
            }
            "app-permissions" => policy.category == AppPermissions,
            "edge" => policy.category == Edge,
            "strict" => true,
            _ => false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn catalog_ids_are_unique_and_kebab_case() {
        let mut seen = BTreeSet::new();
        for policy in all() {
            assert!(seen.insert(policy.id), "duplicate policy ID {}", policy.id);
            assert!(
                policy
                    .id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "{} must be lowercase kebab-case",
                policy.id
            );
            assert!(!policy.id.starts_with('-') && !policy.id.ends_with('-'));
        }
        assert_eq!(all().len(), 65);
    }

    #[test]
    fn registry_locations_are_unique_policy_values() {
        let mut seen = BTreeSet::new();
        for policy in all() {
            assert!(
                seen.insert((policy.key, policy.name)),
                "{} duplicates the registry value of another policy",
                policy.id
            );
            assert!(
                policy.key.starts_with("SOFTWARE\\Policies\\Microsoft\\"),
                "{} must live under the policies hive",
                policy.id
            );
            assert!(!policy.key.ends_with('\\'), "{}", policy.id);
            assert!(!policy.key.starts_with('\\'), "{}", policy.id);
            assert!(
                !policy.name.is_empty() && !policy.name.contains('\\'),
                "{}",
                policy.id
            );
            assert_eq!(
                policy.registry_path(),
                format!("HKLM\\{}\\{}", policy.key, policy.name)
            );
        }
    }

    #[test]
    fn every_policy_is_documented() {
        for policy in all() {
            assert!(
                policy.source.starts_with("https://learn.microsoft.com/"),
                "{} needs a Microsoft reference",
                policy.id
            );
            assert!(
                policy.impact.ends_with('.'),
                "{} impact must be a sentence",
                policy.id
            );
            assert!(policy.impact.len() > 20, "{}", policy.id);
            assert!(!policy.title.is_empty(), "{}", policy.id);
            assert!(
                policy.title.len() <= 40,
                "{} title too long for the UI",
                policy.id
            );
        }
    }

    #[test]
    fn app_permission_policies_force_deny() {
        for policy in in_category(AppPermissions) {
            assert_eq!(policy.key, APP_PRIVACY, "{}", policy.id);
            assert_eq!(policy.desired, 2, "{} must force-deny", policy.id);
            assert!(policy.id.starts_with("app-"), "{}", policy.id);
        }
        assert_eq!(in_category(AppPermissions).count(), 19);
    }

    #[test]
    fn original_seven_policies_are_unchanged() {
        let expected = [
            (
                "advertising-id",
                "SOFTWARE\\Policies\\Microsoft\\Windows\\AdvertisingInfo",
                "DisabledByGroupPolicy",
                1,
            ),
            ("clipboard-sync", SYSTEM, "AllowCrossDeviceClipboard", 0),
            ("activity-feed", SYSTEM, "EnableActivityFeed", 0),
            (
                "online-speech",
                INPUT_PERSONALIZATION,
                "AllowInputPersonalization",
                0,
            ),
            ("app-camera", APP_PRIVACY, "LetAppsAccessCamera", 2),
            ("app-microphone", APP_PRIVACY, "LetAppsAccessMicrophone", 2),
            ("app-location", APP_PRIVACY, "LetAppsAccessLocation", 2),
        ];
        for (id, key, name, desired) in expected {
            let policy = get(id).unwrap();
            assert_eq!(policy.key, key, "{id}");
            assert_eq!(policy.name, name, "{id}");
            assert_eq!(policy.desired, desired, "{id}");
        }
    }

    #[test]
    fn every_category_has_policies_and_labels() {
        for category in Category::ALL {
            assert!(in_category(category).count() > 0, "{category:?} is empty");
            assert!(!category.label().is_empty());
            assert!(category.description().ends_with('.'));
        }
        let covered: BTreeSet<Category> = all().iter().map(|p| p.category).collect();
        assert_eq!(covered.len(), Category::ALL.len());
        let mut ordered = Category::ALL.to_vec();
        ordered.sort();
        assert_eq!(ordered, Category::ALL.to_vec(), "ALL must be in enum order");
    }

    #[test]
    fn catalog_is_grouped_by_category_in_order() {
        let order: Vec<Category> = all().iter().map(|p| p.category).collect();
        let mut sorted = order.clone();
        sorted.sort();
        assert_eq!(
            order, sorted,
            "policies must be listed category by category"
        );
    }

    #[test]
    fn get_is_case_sensitive_and_reports_unknown_ids() {
        assert_eq!(get("advertising-id").unwrap().desired, 1);
        let err = get("Advertising-ID").unwrap_err();
        assert!(matches!(err, Error::UnknownPolicy(ref id) if id == "Advertising-ID"));
        assert_eq!(
            err.to_string(),
            "Unknown policy 'Advertising-ID'. Run list for valid IDs."
        );
    }

    #[test]
    fn select_deduplicates_and_preserves_order() {
        let selected = select(["app-camera", "advertising-id", "app-camera"]).unwrap();
        let ids: Vec<&str> = selected.iter().map(|p| p.id).collect();
        assert_eq!(ids, ["app-camera", "advertising-id"]);
    }

    #[test]
    fn select_rejects_empty_and_unknown() {
        assert!(matches!(
            select(Vec::<String>::new()),
            Err(Error::EmptySelection)
        ));
        assert!(matches!(
            select(["advertising-id", ""]),
            Err(Error::UnknownPolicy(_))
        ));
        assert!(matches!(select(["nope"]), Err(Error::UnknownPolicy(_))));
    }

    #[test]
    fn contains_requires_an_exact_catalog_entry() {
        let real = get("app-location").unwrap();
        assert!(contains(real));
        let forged = PrivacyPolicy {
            key: "SOFTWARE\\Policies\\Microsoft\\Windows\\Evil",
            ..*real
        };
        assert!(!contains(&forged));
        let retargeted = PrivacyPolicy {
            desired: 0,
            ..*real
        };
        assert!(!contains(&retargeted));
    }

    #[test]
    fn high_impact_policies_are_the_disruptive_ones() {
        let high: Vec<&str> = all()
            .iter()
            .filter(|p| p.impact_level == High)
            .map(|p| p.id)
            .collect();
        assert_eq!(
            high,
            [
                "online-speech",
                "location-platform",
                "app-camera",
                "app-microphone",
                "app-location",
                "app-background",
                "app-screen-capture",
                "cloud-notifications",
            ]
        );
        assert_eq!(ImpactLevel::High.label(), "High impact");
        assert!(Moderate < High);
    }

    #[test]
    fn presets_expand_to_non_empty_catalog_subsets() {
        assert_eq!(presets().len(), 4);
        let mut ids = BTreeSet::new();
        for preset in presets() {
            assert!(ids.insert(preset.id), "duplicate preset {}", preset.id);
            let expanded = expand_preset(preset);
            assert!(!expanded.is_empty(), "{} is empty", preset.id);
            assert!(preset.description.ends_with('.'));
            let mut seen = BTreeSet::new();
            for policy in &expanded {
                assert!(
                    seen.insert(policy.id),
                    "{} repeats {}",
                    preset.id,
                    policy.id
                );
                assert!(contains(policy));
            }
            // Expansion is always a valid explicit selection.
            let reselected = select(expanded.iter().map(|p| p.id)).unwrap();
            assert_eq!(reselected, expanded);
        }
        assert_eq!(expand_preset(preset("strict").unwrap()).len(), all().len());
        assert_eq!(expand_preset(preset("app-permissions").unwrap()).len(), 19);
        assert_eq!(expand_preset(preset("edge").unwrap()).len(), 4);
    }

    #[test]
    fn baseline_preset_never_includes_high_impact_or_app_permissions() {
        let baseline = expand_preset(preset("baseline").unwrap());
        assert!(baseline.len() >= 30);
        for policy in &baseline {
            assert_eq!(policy.impact_level, Moderate, "{}", policy.id);
            assert_ne!(policy.category, AppPermissions, "{}", policy.id);
            assert_ne!(policy.category, Edge, "{}", policy.id);
        }
        assert!(baseline.iter().any(|p| p.id == "advertising-id"));
        assert!(baseline.iter().any(|p| p.id == "diagnostic-data"));
        assert!(!baseline.iter().any(|p| p.id == "location-platform"));
    }

    #[test]
    fn unknown_presets_are_rejected() {
        let err = preset("everything").unwrap_err();
        assert!(matches!(err, Error::UnknownPreset(ref id) if id == "everything"));
        assert!(err.to_string().contains("Unknown preset 'everything'"));
        let unlisted = Preset {
            id: "custom",
            title: "Custom",
            description: "Not in the table.",
        };
        assert!(expand_preset(&unlisted).is_empty());
    }

    #[test]
    fn json_shape_matches_the_documented_fields() {
        let json = serde_json::to_value(get("advertising-id").unwrap()).unwrap();
        for field in [
            "Id",
            "Title",
            "Category",
            "Key",
            "Name",
            "Desired",
            "Impact",
            "ImpactLevel",
            "Source",
        ] {
            assert!(json.get(field).is_some(), "missing {field}");
        }
        assert_eq!(json["Desired"], 1);
        assert_eq!(json["Category"], "Personalization");
        let preset_json = serde_json::to_value(preset("baseline").unwrap()).unwrap();
        assert_eq!(preset_json["Id"], "baseline");
    }
}
