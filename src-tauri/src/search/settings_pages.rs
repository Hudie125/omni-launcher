use super::SearchResultItem;

pub struct SettingsPagesScanner;

impl SettingsPagesScanner {
    pub fn search(query: &str) -> Vec<SearchResultItem> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let pages = [
            ("Display Settings", "ms-settings:display", "Resolution, refresh rate, scaling, HDR"),
            ("Sound Settings", "ms-settings:sound", "Volume, audio output, microphone input"),
            ("Notifications & Actions", "ms-settings:notifications", "Do Not Disturb, alerts, badges"),
            ("Power & Sleep", "ms-settings:powersleep", "Screen turn off timer, sleep mode"),
            ("Bluetooth & Devices", "ms-settings:bluetooth", "Manage wireless controllers, mice, headsets"),
            ("Wi-Fi Settings", "ms-settings:network-wifi", "Available wireless networks, properties"),
            ("VPN Connections", "ms-settings:network-vpn", "Configure and connect to virtual private networks"),
            ("Installed Apps", "ms-settings:appsfeatures", "Manage and uninstall desktop software"),
            ("Startup Apps", "ms-settings:startupapps", "Enable or disable apps that launch on boot"),
            ("Windows Update", "ms-settings:windowsupdate", "Check for OS security updates and patches"),
            ("Privacy & Security", "ms-settings:privacy", "Permissions, camera, location access"),
            ("Mouse & Touchpad", "ms-settings:mousetouchpad", "Cursor speed, scrolling, gestures"),
            ("Keyboard Settings", "ms-settings:keyboard", "Key layout, repeat rate, language input"),
            ("Region & Language", "ms-settings:regionlanguage", "System locale, date/time format, keyboards"),
            ("Storage Sense", "ms-settings:storagesense", "Drive usage, temporary file cleanup"),
            ("Camera Privacy", "ms-settings:camera", "Application camera permissions and video preview"),
            ("Microphone Privacy", "ms-settings:microphone", "App microphone access and input level"),
            ("Network & Internet", "ms-settings:network", "Ethernet, proxies, data usage"),
        ];

        let mut results = Vec::new();

        for (title, uri, desc) in pages {
            let title_lower = title.to_lowercase();
            let uri_lower = uri.to_lowercase();
            let desc_lower = desc.to_lowercase();

            if title_lower.contains(&q) || uri_lower.contains(&q) || desc_lower.contains(&q) {
                results.push(SearchResultItem {
                    id: format!("settings_{}", uri),
                    title: format!("[Settings] {}", title),
                    subtitle: Some(desc.to_string()),
                    category: "Settings".into(),
                    item_type: "cmd".into(),
                    card_type: None,
                    path: Some(uri.to_string()),
                    action_hint: Some("↵ Open Settings".into()),
                    score: 85,
                    snippet: None,
                    admin_path: None,
                });
            }
        }

        results
    }
}
